#!/usr/bin/env python3
"""RT3 oracle driver: launch the game with the working config from a single
ancestor, drive it, and dump memory at stages (ptrace_scope-safe).

Usage: python3 tools/oracle/drive.py <stage-file>
The stage file lists one action per line:
  wait <seconds>
  click <x> <y>          (virtual-desktop coordinates)
  key <keysym>           (xdotool key name, e.g. Return, Escape, Down)
  type <text>
  shot <tag>             (screenshot to /tmp/opencode/shot-<tag>.xwd)
  dump <tag>             (memory dump to /tmp/opencode/mem-<tag>/)
"""
import os, subprocess, sys, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import capture_lock

GAME = os.path.expanduser("~/tools/wine-rt3/drive_c/RT3")
PREFIX = os.path.expanduser("~/tools/wine-rt3")
XD = os.path.expanduser("~/tools/xdotool/bin/xdotool")
XWD = os.path.expanduser("~/tools/xwd-pkg/usr/bin/xwd")
OUT = "/tmp/opencode"

ENV = dict(
    os.environ,
    WINEPREFIX=PREFIX,
    WINEDEBUG="err+all",
    WINEDLLOVERRIDES="d3d8=n,d3d9=n,binkw32=",
    DISPLAY=":99",
)

def ensure_display():
    if not os.path.exists("/tmp/.X11-unix/X99"):
        subprocess.Popen(["Xvfb", ":99", "-screen", "0", "1280x1024x24", "+extension", "GLX",
                          "+render", "-dpi", "96"],
                         stdout=open(f"{OUT}/xvfb.log", "wb"), stderr=subprocess.STDOUT)
        time.sleep(2)

def game_pids():
    out = subprocess.run(["pgrep", "-f", "RT3[.]exe"], capture_output=True, text=True).stdout.split()
    pids = []
    for pid in out:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as f:
                if b"RT3.exe" in f.read():
                    pids.append(int(pid))
        except FileNotFoundError:
            pass
    return pids

def become_subreaper():
    """PR_SET_CHILD_SUBREAPER=36: orphans in the launched game tree reparent
    to this process (not to an outer subreaper), so this driver stays a real
    ancestor of RT3.exe and Yama (ptrace_scope=1) allows /proc/pid/mem reads
    (the game is ELF32, so the earlier x86_64 LD_PRELOAD workaround could
    never load into it)."""
    import ctypes
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "prctl(PR_SET_CHILD_SUBREAPER)")


def dump(tag):
    for pid in game_pids():
        dest = f"{OUT}/mem-{tag}"
        os.makedirs(dest, exist_ok=True)
        n = 0
        with open(f"/proc/{pid}/maps") as f:
            for line in f:
                parts = line.split()
                rng, perms = parts[0], parts[1]
                if "rw" not in perms:
                    continue
                lo, hi = (int(x, 16) for x in rng.split("-"))
                mb = (hi - lo) / 1048576
                name = parts[5] if len(parts) > 5 else "anon"
                if mb >= 3 and name == "anon":
                    try:
                        with open(f"/proc/{pid}/mem", "rb", buffering=0) as mem:
                            mem.seek(lo)
                            data = mem.read(hi - lo)
                        with open(f"{dest}/{rng.replace('-', '_')}.bin", "wb") as o:
                            o.write(data)
                        n += 1
                    except OSError:
                        pass
        print(f"dump[{tag}] pid {pid}: {n} regions -> {dest}", flush=True)

def main():
    # Coordination protocol: hold the capture lock for the whole run, from
    # display boot until the game exits (see capture_lock.py).
    lock = capture_lock.acquire()
    become_subreaper()
    ensure_display()
    child = subprocess.Popen(
        ["wine", "explorer", "/desktop=RT3,1024x768", "RT3.exe"],
        cwd=GAME, env=ENV,
        stdout=open(f"{OUT}/rt3-oracle.log", "wb"), stderr=subprocess.STDOUT,
    )
    print(f"launched ancestor pid {child.pid}", flush=True)
    run_actions(open(sys.argv[1]).read().splitlines(), lock=lock)


def take_shot(path):
    """Captures the game window when findable, else the root (the game
    window id changes per run and root misses the game's content)."""
    try:
        wid = subprocess.run([XD, "search", "--name", "Railroad"],
                             env=ENV, capture_output=True, text=True).stdout.split()[0]
        subprocess.run([XWD, "-id", wid, "-out", path], env=ENV)
        return
    except Exception:
        pass
    subprocess.run([XWD, "-root", "-out", path], env=ENV)


def run_actions(lines, lock=None):
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        cmd, *args = line.split()
        if cmd == "wait":
            time.sleep(float(args[0]))
            print(f"waited {args[0]}s; pids={game_pids()}", flush=True)
        elif cmd == "click":
            subprocess.run([XD, "mousemove", args[0], args[1], "click", "1"], env=ENV)
            print(f"click {args}", flush=True)
            time.sleep(1.5)
        elif cmd == "key":
            subprocess.run([XD, "key", "--clearmodifiers", args[0]], env=ENV)
            print(f"key {args}", flush=True)
            time.sleep(1.5)
        elif cmd == "type":
            subprocess.run([XD, "type", "--", " ".join(args)], env=ENV)
            time.sleep(1.0)
        elif cmd == "shot":
            take_shot(f"{OUT}/shot-{args[0]}.xwd")
            print(f"shot {args[0]}", flush=True)
        elif cmd == "dump":
            dump(args[0])
        else:
            print(f"unknown stage action: {line}", flush=True)
    print("stages complete; holding capture; remote control in", f"{OUT}/rt3-rc/", flush=True)
    if lock is not None:
        rc_hold(lock, game_pids)


def rc_hold(lock, pids_of, poll_seconds=2):
    """Holds the capture (and lock) until the game exits, executing remote
    control command files placed in /tmp/opencode/rt3-rc/ (stage grammar;
    this process is the memory-dump-capable ancestor of the game)."""
    import time as _t
    rc_dir = f"{OUT}/rt3-rc"
    os.makedirs(rc_dir, exist_ok=True)
    while True:
        for name in sorted(os.listdir(rc_dir)):
            path = os.path.join(rc_dir, name)
            try:
                lines = open(path).read().splitlines()
            except OSError:
                continue
            os.unlink(path)
            print(f"rc {name}: {lines}", flush=True)
            run_actions(lines)
        if not pids_of():
            print("game exited; capture complete", flush=True)
            return
        _t.sleep(poll_seconds)

if __name__ == "__main__":
    main()
