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
CARGO="${CARGO:-cargo}"

if [ "$mode" != "check" ] && [ "$mode" != "regen" ]; then
    echo "usage: evidence.sh {check|regen}" >&2
    exit 2
fi

if [ -z "$RT3_HOME" ]; then
    echo "RT3_HOME unset; skipping evidence-$mode"
    exit 0
fi

if ! command -v "$CARGO" >/dev/null 2>&1; then
    echo "cargo not found (tried CARGO=$CARGO)" >&2
    exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd) || exit 2
cd "$repo_root" || exit 2

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
exit "$status"
