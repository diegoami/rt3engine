#!/bin/sh
# RT3 oracle launcher — capture-locked (coordination protocol 2026-11-23).
#
# The capture lock is shared with the onenil-archaeology DOSBox-X captures:
# exclusive flock on ~/projects/onenil-archaeology/.cache/capture.lock,
# held for the whole capture (display boot -> game exit), auto-released.
# Never delete the lock file; the protocol is advisory and only works when
# every capture session honors it.
#
# Usage:
#   sh tools/oracle/launch.sh [stage-file]     (default: menu screenshot)
# The stage file lists actions for tools/oracle/drive.py:
#   wait <s> | click <x> <y> | key <k> | type <t> | shot <tag> | dump <tag>
#
# Working config: DXVK x32 d3d8/d3d9 on lavapipe (passes RT3's D3D8.1
# hardware check), binkw32 disabled, Wine virtual desktop, game copy at
# C:/RT3 in WINEPREFIX ~/tools/wine-rt3 (original install untouched).
# Display: renamed Xvfb binary (~/tools/bin/xvfb-rt3) with GLX — renamed
# because legacy cleanup scripts on this box pattern-match "Xvfb".
set -u
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
STAGES="${1:-}"

if [ -z "$STAGES" ]; then
  STAGES="$(mktemp /tmp/opencode/rt3-stages.XXXXXX)"
  printf 'wait 40\nshot menu\n' > "$STAGES"
fi

# The lock is acquired INSIDE drive.py (tools/oracle/capture_lock.py) and
# held until the game exits — do not wrap this in shell flock too: two
# flock file descriptions in one process conflict and would self-deadlock.
exec python3 "$ROOT/tools/oracle/drive.py" "$STAGES"
