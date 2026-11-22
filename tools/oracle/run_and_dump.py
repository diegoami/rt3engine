#!/usr/bin/env python3
"""RT3 oracle runner: spawn the game under Wine as OUR child (so ptrace
works), wait, then dump large anonymous rw regions to /tmp/opencode/mem/.

Usage: python3 tools/oracle/run_and_dump.py [seconds-to-wait]
"""
import os, subprocess, sys, time

GAME = "/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3"
PREFIX = os.path.expanduser("~/tools/wine-rt3")
OUT = "/tmp/opencode/mem"

def main():
    wait = float(sys.argv[1]) if len(sys.argv) > 1 else 25.0
    os.makedirs(OUT, exist_ok=True)
    env = dict(os.environ, WINEPREFIX=PREFIX, WINEDEBUG="-all", DISPLAY=":99")

    if os.system("xdpyinfo -display :99 >/dev/null 2>&1") != 0:
        subprocess.Popen(["Xvfb", ":99", "-screen", "0", "1280x1024x24"],
                         stdout=open("/tmp/opencode/xvfb.log", "wb"),
                         stderr=subprocess.STDOUT)
        time.sleep(2)

    child = subprocess.Popen(["wine", "RT3.exe"], cwd=GAME, env=env,
                             stdout=open("/tmp/opencode/rt3-run.log", "wb"),
                             stderr=subprocess.STDOUT)
    print(f"spawned wine pid {child.pid}", flush=True)
    try:
        time.sleep(wait)
    except KeyboardInterrupt:
        pass

    # the actual RT3 process is a grandchild (wine loader execs it);
    # find it via our session's wineserver children
    pids = subprocess.run(["pgrep", "-f", "RT3.exe"], capture_output=True, text=True).stdout.split()
    print("candidates:", pids, flush=True)
    dumped = 0
    for pid in pids:
        pid = int(pid)
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as f:
                cmd = f.read()
            if b"RT3.exe" not in cmd:
                continue
            with open(f"/proc/{pid}/maps") as f:
                maps = f.read().splitlines()
            for line in maps:
                parts = line.split()
                rng, perms = parts[0], parts[1]
                if "rw" not in perms:
                    continue
                lo, hi = (int(x, 16) for x in rng.split("-"))
                mb = (hi - lo) / 1048576
                name = parts[5] if len(parts) > 5 else "anon"
                if mb >= 3 and name == "anon":
                    out = f"{OUT}/{rng.replace('-', '_')}.bin"
                    try:
                        with open(f"/proc/{pid}/mem", "rb", buffering=0) as mem:
                            mem.seek(lo)
                            data = mem.read(hi - lo)
                        with open(out, "wb") as o:
                            o.write(data)
                        print(f"dumped {out} {mb:.0f}MB", flush=True)
                        dumped += 1
                    except OSError as e:
                        print(f"skip {rng}: {e}", flush=True)
        except FileNotFoundError:
            continue
    print(f"regions dumped: {dumped}", flush=True)
    # leave the game running for further snapshots
    print(f"game still running (pid {child.pid}); pkill RT3.exe to stop", flush=True)

if __name__ == "__main__":
    main()
