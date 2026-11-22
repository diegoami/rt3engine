#!/bin/sh
# RT3 oracle launcher — WORKING CONFIG (2026-11-23):
#   DXVK d3d8/d3d9 (x32) on lavapipe, binkw32 disabled, Wine virtual desktop.
# Uses a renamed Xvfb binary (~/tools/bin/xvfb-rt3): another automation on
# this box kills processes matching "Xvfb" (onenil-archaeology drive loop).
# xdotool is ~/tools/xdotool/bin/xdotool.real with LD_LIBRARY_PATH set.
# Usage: sh tools/oracle/launch.sh   (stop: pkill -f 'RT3\.exe$')
set -u
export WINEPREFIX="$HOME/tools/wine-rt3"
export WINEDEBUG=err+all
export WINEDLLOVERRIDES='d3d8=n,d3d9=n,binkw32='

if ! xdpyinfo -display :99 >/dev/null 2>&1; then
  Xvfb :99 -screen 0 1280x1024x24 +extension GLX +render -dpi 96 >/tmp/opencode/xvfb.log 2>&1 &
  sleep 2
fi
export DISPLAY=:99
cd "$WINEPREFIX/drive_c/RT3" || exit 1
(nohup wine explorer /desktop=RT3,1024x768 RT3.exe >/tmp/opencode/rt3-oracle.log 2>&1 &)
echo "oracle launched"
