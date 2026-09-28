#!/usr/bin/env python3
"""Maintain adjacent modification notices without rewriting upstream source."""

import argparse
from pathlib import Path
import subprocess


BASELINE = "6b9826e3aa83b1a5947db50f4332cb9c65f1b340"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    # Compare the working tree as well as committed content; fail on missing history.
    names = (
        subprocess.check_output(
            ["git", "diff", "--name-only", "--diff-filter=M", "-z", BASELINE],
            cwd=repo,
        )
        .decode()
        .split("\0")
    )
    missing = []
    checked = 0
    for name in filter(None, names):
        source = repo / name
        if source.is_symlink() or name.endswith(".license"):
            continue
        notice = source.with_name(source.name + ".license")
        expected = (
            f"Modification notice for {name}\n\n"
            "This file is distributed with modifications in Prisma AIRS Harness\n"
            f"relative to OpenAI Codex commit {BASELINE}.\n"
            "Changes include fork adaptations and selected later upstream updates.\n"
            "See Git history for individual authors and changes. Original copyright\n"
            "and license notices remain applicable; this notice does not relicense\n"
            "third-party content. See the repository LICENSE, NOTICE and LICENSING.md.\n"
        )
        if not notice.exists() or notice.read_text() != expected:
            if args.write:
                notice.write_text(expected)
            else:
                missing.append(name)
        checked += 1
    if missing:
        raise SystemExit(
            "Missing or stale modification notices:\n" + "\n".join(missing)
        )
    print(f"Verified {checked} modified upstream file notices")


if __name__ == "__main__":
    main()
