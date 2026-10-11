#!/usr/bin/env python3
"""A contact sheet of the listing photos, for checking the art by eye.

    python3 tools/sheet.py OUT.png [id ...]
"""
import sys
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
out = sys.argv[1]
names = sys.argv[2:] or sorted(p.stem for p in (ROOT / "web/art/items").glob("*.png"))
tiles = [Image.open(ROOT / f"web/art/items/{n}.png").convert("RGB").resize((284, 213)) for n in names]
cols = 4
rows = (len(tiles) + cols - 1) // cols
sheet = Image.new("RGB", (cols * 288, rows * 217), "white")
for k, t in enumerate(tiles):
    sheet.paste(t, ((k % cols) * 288 + 2, (k // cols) * 217 + 2))
sheet.save(out)
