#!/usr/bin/env python3
"""Capture-lock coordination (protocol agreed 2026-11-23).

Two capture sessions share this machine: this project's Wine/RT3 oracle
and the onenil-archaeology DOSBox-X captures. Both take an exclusive
advisory lock before booting any emulator/display pipeline and hold it for
the whole capture; flock auto-releases on exit, so crashed captures cannot
deadlock the protocol. Never delete the lock file to force entry.

**This applies to reviewers and any reproduction too:** any session that
boots the emulator (oracle runs, capture reproduction in a review) must
take this lock first — the protocol only works if every capture honors it.
Plain gates (cargo test, fmt, clippy, evidence checks without emulator
boot) do not touch the emulator and need no lock. Review prompts for
oracle-touching changes must state this.

Reference implementation: onenil-archaeology runs/T115/drive.py,
commit 4b81aae (task/T115-league-menu).
"""
import fcntl
import os
import sys
import time

LOCK_PATH = "/home/diego/projects/onenil-archaeology/.cache/capture.lock"


def acquire(log=print):
    """Acquires the capture lock (blocking with a log line while waiting).

    Returns the held lock file object; keep it open for the whole capture
    (it releases when the process exits or the file is closed).
    """
    lock_file = open(LOCK_PATH, "w")
    lock_file.write(f"rt3engine oracle pid={os.getpid()}\n")
    lock_file.flush()
    try:
        fcntl.flock(lock_file, fcntl.LOCK_EX | fcntl.LOCK_NB)
        log("capture lock acquired (free)")
    except BlockingIOError:
        log("capture lock held by another session; waiting (blocking flock, "
            "auto-releases when that capture exits)")
        fcntl.flock(lock_file, fcntl.LOCK_EX)
        log("capture lock acquired after waiting")
    return lock_file


def hold_until_game_exit(lock_file, pids_of, poll_seconds=10, log=print):
    """Keeps the capture (and its lock) alive until the game processes exit.

    pids_of: callable returning the live game pids.
    """
    try:
        while True:
            pids = pids_of()
            if not pids:
                log("game exited; capture complete")
                return
            time.sleep(poll_seconds)
    except KeyboardInterrupt:
        log("interrupted; exiting (flock auto-releases)")
        sys.exit(130)
