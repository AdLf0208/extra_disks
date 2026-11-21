#!/usr/bin/env python3
"""Check that the carried loot tables are in 26.3's dialect, not 1.21.1's.

NeoForge 1.21.1 writes a block's drop guard as a pool-level `"conditions": [...]`
array beside `"bonus_rolls"`. 26.3 takes a **singular** `"condition": {...}` and
spells an integral `rolls` as `1`; the array form is evaluated as false, so the
block drops nothing at all. That failure passes `cargo horizon-mod check`,
`fits` and every unit test — only a real break, or this check, sees it.

The reference shape is the prepared 26.3 resources' own
`data/minecraft/loot_table/blocks/acacia_button.json`:

    "pools": [ { "condition": {"type": "minecraft:survives_explosion"},
                 "entries": [...], "rolls": 1 } ]

    python3 tools/data_shape.py <package.hmod> [more.hmod]
    python3 tools/data_shape.py --data <dir> [more dirs/files]

Exit 0 when every carried loot table is in the 26.3 dialect, 1 otherwise,
printing each offender with the value it found.
"""
from __future__ import annotations

import json
import sys
import zipfile
from pathlib import Path

# Fields 1.21.1 writes that 26.3 does not read.
BANNED = ("conditions", "bonus_rolls")


def pool_offenders(pool: dict) -> list[str]:
    """Every 1.21.1 spelling left in one loot pool."""
    bad = [f'"{key}": {json.dumps(pool[key])}' for key in BANNED if key in pool]
    condition = pool.get("condition")
    if condition is not None and not isinstance(condition, dict):
        bad.append(f'"condition" is not an object: {json.dumps(condition)}')
    if "rolls" in pool and isinstance(pool["rolls"], float):
        bad.append(f'"rolls": {json.dumps(pool["rolls"])} (26.3 spells an integral roll 1)')
    return bad


def entry_offenders(entry: dict) -> list[str]:
    """The same array form one level down, inside a pool entry."""
    return [f'entry "conditions": {json.dumps(entry["conditions"])}'
            for entry in entry.get("entries") or []
            if isinstance(entry, dict) and "conditions" in entry]


def check(name: str, table, bad: list[str]) -> None:
    pools = table.get("pools") if isinstance(table, dict) else None
    if not isinstance(pools, list):
        bad.append(f"{name}: no pools list")
        return
    found: list[str] = []
    for index, pool in enumerate(pools):
        if not isinstance(pool, dict):
            found.append(f"pool {index} is not an object")
            continue
        found += [f"pool {index}: {line}" for line in pool_offenders(pool)]
        found += [f"pool {index}: {line}" for line in entry_offenders(pool)]
    if found:
        bad += [f"{name}: 1.21.1 loot table", *[f"  {line}" for line in found]]
    else:
        print(f"  ok   {name}: 26.3 loot dialect (singular condition, rolls 1)")


def read(name: str) -> list[tuple[str, str]]:
    """(name, text) for every loot table under a package or a data dir."""
    found: list[tuple[str, str]] = []
    if name.endswith(".hmod"):
        with zipfile.ZipFile(name) as archive:
            for entry in sorted(archive.namelist()):
                if entry.startswith("data/") and "/loot_table/" in entry \
                        and entry.endswith(".json"):
                    found.append((f"{Path(name).name}:{entry}", archive.read(entry).decode()))
        return found
    root = Path(name)
    files = sorted(root.rglob("*.json")) if root.is_dir() else [root]
    for path in files:
        if "loot_table" in str(path):
            found.append((str(path), path.read_text()))
    return found


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__.splitlines()[-1], file=sys.stderr)
        return 64
    targets = argv[1:] if argv[0] == "--data" else argv
    bad: list[str] = []
    checked = 0
    for target in targets:
        for name, text in read(target):
            try:
                parsed = json.loads(text)
            except json.JSONDecodeError as error:
                bad.append(f"{name}: not JSON ({error})")
                continue
            checked += 1
            check(name, parsed, bad)
    if bad:
        for line in bad:
            print(f"  FAIL {line}", file=sys.stderr)
        return 1
    print(f"data shape: {checked} loot table(s), every one in the 26.3 dialect")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
