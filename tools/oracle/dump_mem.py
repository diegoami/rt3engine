#!/usr/bin/env python3
"""Dump interesting rw memory regions of the RT3 wine process to /tmp/opencode/."""
import os, subprocess, sys

pids = subprocess.run(['pgrep', '-f', 'RT3.exe'], capture_output=True, text=True).stdout.split()
pids = [p for p in pids if not os.path.exists(f"/proc/{p}/cmdline") or b'bash' not in open(f"/proc/{p}/cmdline","rb").read()]
if not pids:
    sys.exit("no RT3 process")
pid = int(pids[0])
print("pid", pid)
os.makedirs("/tmp/opencode/mem", exist_ok=True)
with open(f"/proc/{pid}/maps") as f:
    for line in f:
        parts = line.split()
        rng, perms = parts[0], parts[1]
        if 'rw' not in perms:
            continue
        lo, hi = (int(x, 16) for x in rng.split('-'))
        mb = (hi - lo) / 1048576
        name = parts[5] if len(parts) > 5 else 'anon'
        if mb >= 3 and 'anon' in name:
            out = f"/tmp/opencode/mem/{rng.replace('-','_')}.bin"
            try:
                with open(f"/proc/{pid}/mem", "rb", buffering=0) as mem:
                    mem.seek(lo)
                    data = mem.read(hi - lo)
                with open(out, "wb") as o:
                    o.write(data)
                print(f"dumped {out} {mb:.0f}MB")
            except OSError as e:
                print(f"skip {rng}: {e}")
