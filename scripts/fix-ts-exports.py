#!/usr/bin/env python3
"""
Rewrites bare #[ts(export)] attributes in SeaORM-generated entity files to
#[ts(export, export_to = "<stem>.ts")] so ts-rs emits each type into its own file.

Usage:
    python3 scripts/fix-ts-exports.py [entities_dir]

Defaults to kernel/src/entities/postgres relative to the project root,
or accepts an explicit path (useful when invoked from inside kernel/).
"""

import re
import sys
from pathlib import Path

PATTERN = re.compile(r'#\[ts\(export\)\]')


def fix_file(path: Path) -> bool:
    stem = path.stem
    replacement = f'#[ts(export, export_to = "{stem}.ts")]'
    original = path.read_text(encoding="utf-8")
    updated = PATTERN.sub(replacement, original)
    if updated == original:
        return False
    path.write_text(updated, encoding="utf-8")
    return True


def main() -> None:
    entities_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("src/entities/postgres")

    if not entities_dir.exists():
        print(f"error: directory not found: {entities_dir}", file=sys.stderr)
        sys.exit(1)

    changed = [f for f in sorted(entities_dir.glob("*.rs")) if fix_file(f)]

    if changed:
        for f in changed:
            print(f"  patched: {f}")
    else:
        print("  nothing to patch (no bare #[ts(export)] found)")


if __name__ == "__main__":
    main()
