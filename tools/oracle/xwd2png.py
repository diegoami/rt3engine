#!/usr/bin/env python3
"""xwd2png.py — convert an XWD screenshot (24bpp ZPixmap) to PNG (stdlib only)."""
import struct, sys, zlib

def main():
    src, dst = sys.argv[1], sys.argv[2]
    d = open(src, "rb").read()
    (hs, ver, fmt, depth, w, h, xoff, byte_order, bitmap_unit,
     bitmap_bit_order, bitmap_pad, bpp, bpl) = struct.unpack_from(">13I", d, 0)
    data = d[hs:]
    if bpp != 24:
        sys.exit(f"unsupported bpp {bpp}")
    # build PNG
    def chunk(tag, payload):
        c = struct.pack(">I", len(payload)) + tag + payload
        return c + struct.pack(">I", zlib.crc32(tag + payload) & 0xFFFFFFFF)
    raw = bytearray()
    for y in range(h):
        raw.append(0)  # filter none
        raw += data[y*bpl : y*bpl + w*3]
    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(bytes(raw), 6))
    png += chunk(b"IEND", b"")
    open(dst, "wb").write(png)
    print(f"{dst}: {w}x{h}")

main()
