#!/usr/bin/env python3
"""Every item in data/items.json has its listing photo, and the scenes and
icon the page names exist (tools/art.py makes them)."""
import json, pathlib, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
items = json.loads((ROOT / "data/items.json").read_text())
need = [f"web/art/items/{i['id']}.png" for i in items] + ["web/art/title.png", "web/art/final.png", "web/art/icon.png"]
missing = [p for p in need if not (ROOT / p).is_file()]
for p in missing:
    print("missing", p)
print(f"{len(need) - len(missing)} of {len(need)} pictures present")
sys.exit(1 if missing else 0)
