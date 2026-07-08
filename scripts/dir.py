from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

IGNORE_DIRS = {
    "target",
    ".git",
    ".github",
    "dist",
    "build",
    "xencommit"
}

EXTENSIONS = {
    ".rs",
    ".wgsl",
    ".toml",
}

for path in sorted(ROOT.rglob("*")):
    if not path.is_file():
        continue

    if any(part in IGNORE_DIRS for part in path.parts):
        continue

    if path.suffix in EXTENSIONS:
        print(path.relative_to(ROOT))