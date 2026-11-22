#!/usr/bin/env python3
"""Click-grid probe: click points on the display, screenshot after each,
report which points changed the screen (the game's real interactive area)."""
import os, struct, subprocess, sys, time

XD = os.path.expanduser("~/tools/xdotool/bin/xdotool.real")
XWD = os.path.expanduser("~/tools/xwd-pkg/usr/bin/xwd")
ENV = dict(os.environ, DISPLAY=":99", LD_LIBRARY_PATH=os.path.expanduser("~/tools/xdotool/lib"))
OUT = "/tmp/opencode"

def shot():
    p = f"{OUT}/grid.xwd"
    subprocess.run([XWD, "-root", "-out", p], env=ENV, capture_output=True)
    d = open(p, "rb").read()
    hs = struct.unpack_from(">I", d, 0)[0]
    w = struct.unpack_from(">I", d, 16)[0]
    h = struct.unpack_from(">I", d, 20)[0]
    bpl = struct.unpack_from(">I", d, 48)[0]
    return d[hs:], w, h, bpl

def diff(a, b, w, h, bpl):
    n = 0
    for y in range(0, h, 10):
        for x in range(0, w, 20):
            base = y * bpl + x * 4
            la = (a[base]*299 + a[base+1]*587 + a[base+2]*114) // 1000
            lb = (b[base]*299 + b[base+1]*587 + b[base+2]*114) // 1000
            if abs(la - lb) > 30:
                n += 1
    return n

def main():
    base, w, h, bpl = shot()
    print("baseline captured", flush=True)
    for y in range(80, 640, 110):
        for x in range(100, 1250, 190):
            subprocess.run([XD, "mousemove", str(x), str(y), "click", "1"], env=ENV, capture_output=True)
            time.sleep(2.0)
            cur, _, _, _ = shot()
            d = diff(base, cur, w, h, bpl)
            if d > 0:
                print(f"CLICK ({x},{y}) -> changed {d} points", flush=True)
                base = cur  # new baseline
            else:
                sys.stdout.write(".")
                sys.stdout.flush()
    print("\ngrid complete", flush=True)

main()
