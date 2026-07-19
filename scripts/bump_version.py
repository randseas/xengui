#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0

from __future__ import annotations

import argparse
import pathlib
import re
import sys

VERSION_RE = re.compile(r"^\d+\.\d+\.\d+$")


def validate_version(version: str):
    if not VERSION_RE.fullmatch(version):
        raise ValueError(
            f"Invalid version '{version}'. Expected format: MAJOR.MINOR.PATCH"
        )


def patch_cargo(text: str, old: str, new: str):
    pattern = re.compile(
        rf'(^version\s*=\s*"){re.escape(old)}(")',
        re.MULTILINE,
    )
    return pattern.sub(rf"\g<1>{new}\2", text)


def patch_readme(text: str, old: str, new: str):
    return text.replace(old, new)


def collect_target(target: pathlib.Path) -> list[pathlib.Path]:
    target = target.resolve()

    if not target.exists():
        raise FileNotFoundError(target)

    files = []

    if target.is_file():
        if target.suffix == ".lock":
            raise RuntimeError(f"Refusing to modify lock file: {target}")

        if target.name not in ("Cargo.toml", "README.md"):
            raise RuntimeError(
                f"Unsupported file: {target}\n"
                "Only Cargo.toml and README.md are supported."
            )

        files.append(target)
        return files

    for path in target.rglob("*"):
        if not path.is_file():
            continue

        if path.suffix == ".lock":
            continue

        if path.name in ("Cargo.toml", "README.md"):
            files.append(path.resolve())

    return files


def main():
    parser = argparse.ArgumentParser(
        description="Safely bump versions in Cargo.toml and README.md."
    )

    parser.add_argument(
        "targets",
        nargs="+",
        help="Files or directories to scan.",
    )

    parser.add_argument("--old", required=True)
    parser.add_argument("--new", required=True)
    parser.add_argument("--dry-run", action="store_true")

    args = parser.parse_args()

    validate_version(args.old)
    validate_version(args.new)

    files = []

    for target in args.targets:
        files.extend(collect_target(pathlib.Path(target)))

    # Remove duplicates while preserving order.
    files = list(dict.fromkeys(files))

    if not files:
        sys.exit("No Cargo.toml or README.md files found.")

    changes = []

    for path in files:
        text = path.read_text(encoding="utf-8")

        if path.name == "Cargo.toml":
            new_text = patch_cargo(text, args.old, args.new)
        else:
            new_text = patch_readme(text, args.old, args.new)

        if new_text != text:
            changes.append((path, new_text))

    if not changes:
        sys.exit(f"No occurrences of {args.old} were found.")

    print("Files to update:")

    for path, _ in changes:
        print(" -", path)

    if args.dry_run:
        print("\nDry run complete.")
        return

    for path, new_text in changes:
        path.write_text(new_text, encoding="utf-8")

    print(f"\nUpdated {len(changes)} file(s).")


if __name__ == "__main__":
    main()
