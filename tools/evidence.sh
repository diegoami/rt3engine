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
    files_matches=$(printf '%s\n' "$line" | tr -d ',' | grep -Eo '[0-9]+ files' || true)
    gb_matches=$(printf '%s\n' "$line" | grep -Eo '[0-9]+\.[0-9]+ GB' || true)
    [ -n "$files_matches" ] && [ -n "$gb_matches" ] || return 1
    [ "$(printf '%s\n' "$files_matches" | wc -l | tr -d ' ')" = 1 ] || return 1
    [ "$(printf '%s\n' "$gb_matches" | wc -l | tr -d ' ')" = 1 ] || return 1
    [ "${files_matches%% *}" = "$want_files" ] || return 1
    awk -v a="${gb_matches%% *}" -v b="$want_gb" 'BEGIN { exit (a == b ? 0 : 1) }'
}

# check_doc_totals_file DOC FILES GB — every row in DOC that names the install
# artifact must state exactly these totals; a missing row is drift too, so a
# stale duplicate row cannot hide behind a correct one.
check_doc_totals_file() {
    doc=$1
    want_files=$2
    want_gb=$3
    rows=0
    row_drift=0
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        rows=$((rows + 1))
        if ! documented_total_line_matches "$line" "$want_files" "$want_gb"; then
            echo "documented totals drift: $doc row does not state $want_files files and ~$want_gb GB: $line" >&2
            row_drift=1
        fi
    done <<EOF
$(grep -E '(^|[^A-Za-z0-9._~-])install-inventory-v1\.json([^A-Za-z0-9._~-]|$)' "$doc" || true)
EOF
    if [ "$rows" -eq 0 ]; then
        echo "documented totals drift: $doc has no install-inventory-v1.json row" >&2
        row_drift=1
    fi
    return "$row_drift"
}

# selftest_documented_totals guards the matcher and the row walk against stale
# totals: a value that only contains the expected one, and a stale duplicate
# row behind a correct one, must both be rejected. It runs with the check.
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
    if documented_total_line_matches 'Aggregate installation inventory: 1,177 files, ~2.03 GB, previously 1,176 files;' 1177 2.03; then
        echo "documented totals self-test: accepted two file counts on one row" >&2
        ok=1
    fi
    tmp=$(mktemp "${TMPDIR:-/tmp}/evidence-totals.XXXXXX") || return 2
    printf '%s\n%s\n' \
        '| `install-inventory-v1.json` | 1,177 files, ~2.03 GB |' \
        '| `install-inventory-v1.json` | 11,177 files, ~2.03 GB |' > "$tmp"
    if check_doc_totals_file "$tmp" 1177 2.03 2>/dev/null; then
        echo "documented totals self-test: accepted a stale duplicate row" >&2
        ok=1
    fi
    printf '%s\n' 'no install-inventory-v1.json row here' > "$tmp"
    if check_doc_totals_file "$tmp" 1177 2.03 2>/dev/null; then
        echo "documented totals self-test: accepted a missing row" >&2
        ok=1
    fi
    printf '%s\n' '| `install-inventory-v1xjson` | 1,177 files, ~2.03 GB |' > "$tmp"
    if check_doc_totals_file "$tmp" 1177 2.03 2>/dev/null; then
        echo "documented totals self-test: accepted a near-miss artifact name" >&2
        ok=1
    fi
    for lookalike in \
        'install-inventory-v1.json.bak' \
        'install-inventory-v1.json~' \
        'xinstall-inventory-v1.json'; do
        printf '| `%s` | 1,177 files, ~2.03 GB |\n' "$lookalike" > "$tmp"
        if check_doc_totals_file "$tmp" 1177 2.03 2>/dev/null; then
            echo "documented totals self-test: accepted a lookalike artifact name ($lookalike)" >&2
            ok=1
        fi
    done
    printf '%s\n%s\n' \
        '| `install-inventory-v1.json` | 1,177 files, ~2.03 GB |' \
        '| `install-inventory-v1.json` | 1,177 files, ~2.03 GB |' > "$tmp"
    check_doc_totals_file "$tmp" 1177 2.03 2>/dev/null || ok=1
    rm -f "$tmp"
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
        check_doc_totals_file "$doc" "$files" "$gb" || drift=1
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
