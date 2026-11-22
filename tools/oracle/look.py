#!/usr/bin/env python3
"""Look: capture a screenshot and print an ASCII view (correct XWD stride)."""
import struct, subprocess, sys, os

XWD = os.path.expanduser("~/tools/xwd-pkg/usr/bin/xwd")
ENV = dict(os.environ, DISPLAY=":99")

def load(path=None):
    if path is None:
        path = "/tmp/opencode/look.xwd"
        subprocess.run([XWD, "-root", "-out", path], env=ENV, capture_output=True)
    d = open(path, "rb").read()
    hs = struct.unpack_from(">I", d, 0)[0]
    w = struct.unpack_from(">I", d, 16)[0]
    h = struct.unpack_from(">I", d, 20)[0]
    bpl = struct.unpack_from(">I", d, 48)[0]
    return d[hs:], w, h, bpl

def luma(data, bpl, x, y):
    base = y * bpl + x * 4
    return (data[base]*299 + data[base+1]*587 + data[base+2]*114) // 1000

def show(x0=0, x1=None, y0=0, y1=760, sx=12, sy=16, src=None):
    data, w, h, bpl = load(src)
    x1 = min(x1 or w, w)
    y1 = min(y1, h)
    chars = " .:-=+*#%@"
    for by in range(y0, y1 - sy + 1, sy):
        line = []
        for bx in range(x0, x1 - sx + 1, sx):
            s = n = 0
            for y in range(by, by + sy, 4):
                for x in range(bx, bx + sx, 3):
                    s += luma(data, bpl, x, y); n += 1
            line.append(chars[min(9, (s // n) * 10 // 256)])
        print("".join(line))

if __name__ == "__main__":
    args = [int(a) for a in sys.argv[1:]]
    show(*args)
