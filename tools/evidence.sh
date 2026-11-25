#!/bin/sh
# Install-backed evidence regeneration and verification.
#
# Usage: evidence.sh {check|regen}
#
# Requires RT3_HOME (self-skips with status 0 otherwise) and cargo (or CARGO).
# Artifacts are produced into temporary files and only moved into place after
# the producer succeeds, so a failed producer never truncates a committed
# artifact. In check mode every artifact is compared; any drift or producer
# failure exits non-zero after reporting all of them.

set -u

mode="${1:-}"
RT3_HOME="${RT3_HOME:-}"
RT3_SAVE="${RT3_SAVE:-}"
RT3_CONTROL_SAVE="${RT3_CONTROL_SAVE:-}"
RT3_CONTROL_SAVE_2="${RT3_CONTROL_SAVE_2:-}"
CARGO="${CARGO:-cargo}"

if [ "$mode" != "check" ] && [ "$mode" != "regen" ]; then
    echo "usage: evidence.sh {check|regen}" >&2
    exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd) || exit 2
cd "$repo_root" || exit 2

# documented_total_line_matches LINE FILES GB — true when LINE states exactly
# these totals: the whole file count and the whole two-decimal GB size, not a
# value that merely contains them as a substring.
documented_total_line_matches() {
    line=$1
    want_files=$2
    want_gb=$3
    line_files=$(printf '%s\n' "$line" | tr -d ',' | grep -Eo '[0-9]+ files' | awk '{ print $1; exit }')
    line_gb=$(printf '%s\n' "$line" | grep -Eo '[0-9]+\.[0-9]+ GB' | awk '{ print $1; exit }')
    [ -n "$line_files" ] && [ -n "$line_gb" ] &&
        [ "$line_files" = "$want_files" ] &&
        awk -v a="$line_gb" -v b="$want_gb" 'BEGIN { exit (a == b ? 0 : 1) }'
}

# selftest_documented_totals guards the matcher against accepting a stale total
# that only contains the expected one as a substring; it runs with the check.
selftest_documented_totals() {
    ok=0
    documented_total_line_matches 'Aggregate installation inventory: 1,177 files, ~2.03 GB;' 1177 2.03 || ok=1
    if documented_total_line_matches 'Aggregate installation inventory: 11,177 files, ~2.03 GB;' 1177 2.03; then
        echo "documented totals self-test: accepted a stale file count" >&2
        ok=1
    fi
    if documented_total_line_matches 'Aggregate installation inventory: 1,177 files, ~12.03 GB;' 1177 2.03; then
        echo "documented totals self-test: accepted a stale size" >&2
        ok=1
    fi
    if documented_total_line_matches 'Aggregate installation inventory: 1,176 files, ~2.03 GB;' 1177 2.03; then
        echo "documented totals self-test: accepted a stale file count" >&2
        ok=1
    fi
    [ "$ok" -eq 0 ] || echo "documented totals self-test failed" >&2
    return "$ok"
}

# check_documented_totals verifies the install totals that the evidence README
# and the environment doc state in prose against the committed artifact, so a
# refreshed inventory cannot leave a stale number behind. It needs no
# installation, so it also runs in check mode when RT3_HOME is unset (CI).
check_documented_totals() {
    artifact=docs/evidence/install-inventory-v1.json
    totals=$(awk '
        /"total":/ { intotal = 1; next }
        intotal && /"files":/ { line = $0; gsub(/[^0-9]/, "", line); files = line; next }
        intotal && /"bytes":/ { line = $0; gsub(/[^0-9]/, "", line); bytes = line; exit }
        END { print files, bytes }
    ' "$artifact")
    files=${totals%% *}
    bytes=${totals##* }
    if [ -z "$files" ] || [ -z "$bytes" ]; then
        echo "documented totals: cannot read install totals from $artifact" >&2
        return 2
    fi
    gb=$(awk -v b="$bytes" 'BEGIN { printf "%.2f", b / 1000000000 }')
    drift=0
    for doc in docs/evidence/README.md docs/environment.md; do
        line=$(grep -m1 'install-inventory-v1.json' "$doc" || true)
        if [ -z "$line" ]; then
            echo "documented totals drift: $doc has no install-inventory-v1.json row" >&2
            drift=1
            continue
        fi
        if ! documented_total_line_matches "$line" "$files" "$gb"; then
            echo "documented totals drift: $doc does not state $files files and ~$gb GB on its install-inventory-v1.json row" >&2
            drift=1
        fi
    done
    return "$drift"
}

if [ "$mode" = "check" ]; then
    selftest_documented_totals || exit $?
    check_documented_totals || exit $?
fi

if [ -z "$RT3_HOME" ]; then
    echo "RT3_HOME unset; skipping evidence-$mode"
    exit 0
fi

if ! command -v "$CARGO" >/dev/null 2>&1; then
    echo "cargo not found (tried CARGO=$CARGO)" >&2
    exit 2
fi

# process [--release] TARGET PRODUCER-ARGS...
#   0 = ok, 1 = drift (check mode), 2 = producer or environment failure.
#   --release builds the producer with --release (the terrain scan walks
#   every map with a validating scanner and is far too slow in debug).
process() {
    release=""
    if [ "${1:-}" = "--release" ]; then
        release="--release"
        shift
    fi
    target=$1
    shift
    tmp=$(mktemp "${target}.tmp.XXXXXX") || return 2
    if ! "$CARGO" run -q $release -p rt3ctl -- "$@" > "$tmp"; then
        echo "evidence producer failed: rt3ctl $*" >&2
        rm -f "$tmp"
        return 2
    fi
    if [ "$mode" = "check" ]; then
        if cmp -s "$tmp" "$target"; then
            rm -f "$tmp"
            return 0
        fi
        echo "evidence drift: $target (run make evidence-regen)" >&2
        rm -f "$tmp"
        return 1
    fi
    if mv "$tmp" "$target"; then
        echo "regenerated $target"
        return 0
    fi
    return 2
}

status=0
process docs/evidence/install-inventory-v1.json inventory --json || status=$?
process docs/evidence/pk4-archives-v1.json pk4 list --json || status=$?
process docs/evidence/typed-tables-v1.json typed list --json || status=$?
process docs/evidence/maps-inventory-v1.json map list --json || status=$?
process --release docs/evidence/terrain-scan-v1.json map terrain --all --json || status=$?
if [ -n "$RT3_SAVE" ]; then
    process --release docs/evidence/save-inventory-v1.json save inspect --path "$RT3_SAVE" --json || status=$?
else
    echo "RT3_SAVE unset; skipping optional save-inventory-$mode"
fi
if [ -n "$RT3_CONTROL_SAVE" ]; then
    process --release docs/evidence/controlled-save-inventory-v1.json save inspect --path "$RT3_CONTROL_SAVE" --json || status=$?
else
    echo "RT3_CONTROL_SAVE unset; skipping optional controlled-save-inventory-$mode"
fi
if [ -n "$RT3_CONTROL_SAVE" ] && [ -n "$RT3_CONTROL_SAVE_2" ]; then
    process --release docs/evidence/controlled-save-diff-v1.json save diff --path-a "$RT3_CONTROL_SAVE" --path-b "$RT3_CONTROL_SAVE_2" --json || status=$?
else
    echo "RT3_CONTROL_SAVE_2 unset; skipping optional controlled-save-diff-$mode"
fi
exit "$status"
