#!/usr/bin/env python3
"""Lowballer's 2D art, drawn as TikZ and compiled, not generated as pixels
(the TikZ figure prompt in GearMaster2D/tikz_figure_prompt.md, as Long Time
Comin's and Food Truck Circus's pictures are made).

Each listing photo is one item standing on a floor in front of a coloured
wall, with a Corporate Memphis blob and confetti behind it. The title and
final pictures put the items in a street like Sam_Comm_final.png in the
vagrancy repository: a warm yellow sky with outlined clouds, salmon, coral
and pink buildings with rooftop clutter, sage greens, flat fills, and one
plum ink outlining every shape.

    python3 tools/art.py            # every picture: web/art/**.png, tools/art/**.tex
    python3 tools/art.py title      # one, or a folder: python3 tools/art.py items

Facts each picture must get right (the prompt's list of what an image
generator gets wrong), and where this file enforces each:

- text: no words, letters or numbers in any picture. The page sets every
  word from data/copy.json, and this file has no \\node with text.
- one picture per item in data/items.json, named by its id: ITEMS below has
  a key for each, and tests/art.py (run by check.sh) fails on a missing one.
- the listing photos are 4:3 (240 x 180 units), the scenes 16:9 (480 x 270),
  the icon square: SIZES.
- the item is the subject: it stands on the floor line (y = FLOOR) in the
  middle third, and the confetti stays in the corners (`confetti`).
- colour is never the only cue: every filled shape has the plum outline.
- repeatable: every random choice comes from a generator seeded with the
  picture's name, so the same script draws the same pictures.

Coordinates are in picture units with y pointing down, as the item drawings
were laid out; the TikZ y unit is negative to match (see PREAMBLE).
"""
import random, re, shutil, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "web" / "art"
SRC = ROOT / "tools" / "art"

# Sam_Comm_final.png's palette, by eye, as in Long Time Comin's art.py, plus
# the few item colours the street does not have (glass, steel).
COLOURS = {
    "ink": "3B2340", "dark": "4A3357", "darkhi": "5E4A6E",
    "sky": "EFC46A", "skyhi": "F5DA92", "skylow": "E9B060", "cloud": "F2D58A",
    "wallA": "E88A73", "wallB": "EE9F7E", "wallC": "E47A7E", "wallD": "F0B07E", "wallE": "D9765F", "wallF": "E99A8A",
    "greenA": "8DBB7E", "greenB": "6FA77A", "greenC": "A9C97F", "greenD": "5E9C6A",
    "window": "F6E7A0", "windark": "8C5A64",
    "plum": "8A5A9E", "plumhi": "A77BB8", "lilac": "D9C3E0",
    "coral": "E47A7E", "coraldk": "C85F65", "orange": "F26B38", "salmon": "F4BE9C",
    "gold": "F2B53A", "golddk": "D9962A", "butter": "F5DA92", "cream": "FFF4E0", "paper": "F6E7C4",
    "sage": "8DBB7E", "sagedk": "6FA77A", "forest": "5E9C6A", "mint": "C5E0B4",
    "teal": "5FA8A0", "glass": "A9D3C9", "glassdk": "7FB7B0",
    "pink": "F2A7B5", "rosewall": "F3B7A8",
    "steel": "C8C2B4", "steeldk": "8D9A93", "metal": "8D9A93",
    "wood": "B9835A", "wooddk": "8E5E3C", "tan": "E2BD8C", "paving": "E2BD8C", "ground": "D3A77D",
}

# Unit: 0.4 mm per picture unit. y is negative because the drawings below
# count y downward from the top edge, as a screen does.
PREAMBLE = r"""\documentclass[tikz,border=0pt]{standalone}
\usepackage{xcolor}
\usetikzlibrary{calc}
% Palette: Sam_Comm_final.png by eye (vagrancy repository).
""" + "\n".join(f"\\definecolor{{{k}}}{{HTML}}{{{v}}}" for k, v in COLOURS.items()) + r"""
% One ink for every outline.
\newcommand{\Ink}{1.3pt}
\tikzset{inked/.style={draw=ink, line width=\Ink, line join=round, line cap=round}}
% Picture units: 0.4 mm, y counted downward (hence the negative y unit).
\tikzset{x=0.4mm, y=-0.4mm}
"""

SIZES = {"item": (240, 180), "scene": (480, 270), "icon": (64, 64)}
FLOOR = 150


# The scale of the drawing being placed. TikZ scales coordinates but not
# lengths given in mm or pt, so corner radii read it (see `item_at`).
SIZE = [1.0]


def f(x):
    return f"{x:.2f}".rstrip("0").rstrip(".")


def P(x, y):
    return f"({f(x)},{f(y)})"


def style(fill, extra=""):
    s = f"inked,fill={fill}"
    return s + ("," + extra if extra else "")


def R(x, y, w, h, fill, r=0, extra=""):
    rc = f",rounded corners={f(r * 0.4 * SIZE[-1])}mm" if r else ""
    return f"\\filldraw[{style(fill, extra)}{rc}] {P(x, y)} rectangle {P(x + w, y + h)};\n"


def C(x, y, r, fill, extra=""):
    return f"\\filldraw[{style(fill, extra)}] {P(x, y)} circle [radius={f(r)}];\n"


def E(x, y, rx, ry, fill, rot=0, extra=""):
    rotate = f",rotate around={{{f(rot)}:{P(x, y)}}}" if rot else ""
    return f"\\filldraw[{style(fill, extra)}{rotate}] {P(x, y)} ellipse [x radius={f(rx)}, y radius={f(ry)}];\n"


def path(d):
    """An SVG path (M L H V C Q Z, absolute or relative) as a TikZ path."""
    toks = re.findall(r"[MLHVCQZmlhvcqz]|-?\d*\.?\d+", d)
    out, i, cur, start, cmd, first = [], 0, (0.0, 0.0), (0.0, 0.0), None, True

    def num():
        nonlocal i
        v = float(toks[i])
        i += 1
        return v

    while i < len(toks):
        if re.match("[A-Za-z]", toks[i]):
            cmd = toks[i]
            i += 1
            if cmd in "Zz":
                out.append("-- cycle")
                cur = start
                continue
        rel = cmd.islower()
        bx, by = cur if rel else (0.0, 0.0)
        c = cmd.upper()
        if c == "M":
            x, y = num() + bx, num() + by
            out.append(P(x, y))
            cur = start = (x, y)
            cmd = "l" if rel else "L"
            first = False
        elif c == "L":
            x, y = num() + bx, num() + by
            out.append("-- " + P(x, y))
            cur = (x, y)
        elif c == "H":
            x = num() + bx
            out.append("-- " + P(x, cur[1]))
            cur = (x, cur[1])
        elif c == "V":
            y = num() + by
            out.append("-- " + P(cur[0], y))
            cur = (cur[0], y)
        elif c == "C":
            x1, y1, x2, y2 = num() + bx, num() + by, num() + bx, num() + by
            x, y = num() + bx, num() + by
            out.append(f".. controls {P(x1, y1)} and {P(x2, y2)} .. {P(x, y)}")
            cur = (x, y)
        elif c == "Q":
            qx, qy = num() + bx, num() + by
            x, y = num() + bx, num() + by
            c1 = (cur[0] + 2 / 3 * (qx - cur[0]), cur[1] + 2 / 3 * (qy - cur[1]))
            c2 = (x + 2 / 3 * (qx - x), y + 2 / 3 * (qy - y))
            out.append(f".. controls {P(*c1)} and {P(*c2)} .. {P(x, y)}")
            cur = (x, y)
    assert not first
    return " ".join(out)


def S(d, fill, extra=""):
    """A filled, inked shape from an SVG path."""
    return f"\\filldraw[{style(fill, extra)}] {path(d)};\n"


LINE = [1.0]  # line-width factor for the drawing being placed (see `item_at`)


def L(d, colour="ink", w=1.3, extra=""):
    """A stroke with no fill: lines, handles, frames."""
    e = "," + extra if extra else ""
    return f"\\draw[{colour}, line width={f(w * LINE[-1])}pt, line cap=round, line join=round{e}] {path(d)};\n"


def scope(body, x=0, y=0, s=1, rot=0, about=(120, 90), ink=None):
    """Places a drawing: shifted by (x, y), scaled by s, rotated about a point.
    `ink` resets the outline width inside it (a TeX group keeps it local)."""
    opts = [f"shift={{{P(x, y)}}}"]
    if s != 1:
        opts.append(f"scale={f(s)}")
    if rot:
        opts.append(f"rotate around={{{f(rot)}:{P(*about)}}}")
    reset = f"\\renewcommand{{\\Ink}}{{{f(ink)}pt}}\n" if ink else ""
    return f"\\begin{{scope}}[{', '.join(opts)}]\n{reset}{body}\\end{{scope}}\n"


# ---- the items, each in a 240 x 180 box, standing on y = FLOOR ----------------

def toaster():
    return (
        R(80, 40, 36, 50, "wood", 10) + R(85, 45, 26, 42, "gold", 7)
        + R(124, 46, 36, 44, "wood", 10) + R(129, 51, 26, 36, "gold", 7)
        + R(58, 72, 124, 78, "coral", 28)
        + R(70, 82, 100, 7, "pink", 3.5, "draw=none")
        + R(180, 98, 20, 10, "dark", 5)
        + C(96, 112, 8, "cream") + C(120, 112, 5, "dark")
        + R(74, 146, 18, 8, "dark", 3) + R(148, 146, 18, 8, "dark", 3)
    )


def jersey():
    return (
        S("M78 34 L105 26 Q120 42 135 26 L162 34 L196 66 L174 90 L162 80 L162 152 L78 152 L78 80 L66 90 L44 66 Z", "plum")
        + L("M105 26 Q120 42 135 26", "gold", 5)
        + L("M78 80 L78 150 M162 80 L162 150", "plumhi", 3)
        + R(78, 96, 84, 12, "gold", 0)
        + L("M90 76 Q100 62 110 74 Q118 84 126 68 Q132 58 150 70", "ink", 2.2)
        + S("M44 66 L66 90 L72 84 L50 60 Z", "gold") + S("M196 66 L174 90 L168 84 L190 60 Z", "gold")
    )


def sedan():
    return (
        S("M24 118 L44 90 Q56 76 78 74 L150 74 Q170 74 184 90 L204 104 Q218 106 218 122 L218 136 L22 136 L22 124 Z", "teal")
        + S("M66 92 L84 80 L116 80 L116 104 L58 104 Z", "glass")
        + S("M126 80 L150 80 Q162 82 174 104 L126 104 Z", "glass")
        + L("M24 118 L218 118", "ink", 1.3)
        + R(122, 108, 12, 4, "dark", 2)
        + R(204, 110, 14, 8, "gold", 3) + R(20, 112, 8, 8, "coral", 2)
        + C(70, 138, 19, "dark") + C(70, 138, 8, "steel")
        + C(176, 138, 19, "dark") + C(176, 138, 8, "steel")
    )


def bike():
    return (
        L("M68 122 L104 80 L156 80 L172 122 M104 80 L122 122 L156 80 M122 122 L68 122", "orange", 7)
        + f"\\draw[ink, line width={f(5 * LINE[-1])}pt] {P(68, 122)} circle [radius=32];\n"
        + f"\\draw[ink, line width={f(5 * LINE[-1])}pt] {P(172, 122)} circle [radius=32];\n"
        + C(68, 122, 4, "steel") + C(172, 122, 4, "steel") + C(122, 122, 7, "steel")
        + L("M104 80 L100 66", "dark", 5) + R(86, 60, 28, 8, "dark", 4)
        + L("M156 80 L150 58 L168 54", "dark", 5)
    )


def guitar_body(fill, sound_hole):
    body = S("M120 90 C90 86 78 104 86 120 C92 132 80 142 88 156 C98 172 142 172 152 156 C160 142 148 132 154 120 C162 104 150 86 120 90 Z", fill)
    if sound_hole:
        return body + C(120, 124, 10, "dark") + R(108, 146, 24, 6, "dark", 2)
    return body + R(106, 118, 28, 8, "dark", 2) + R(106, 134, 28, 8, "dark", 2) + C(140, 152, 4, "cream") + S("M96 100 Q104 96 110 104 L102 114 Z", "cream")


def guitar():
    body = R(113, 10, 14, 92, "wooddk") + R(109, 0, 22, 22, "dark", 5) + guitar_body("coral", False)
    return scope(body, 12, 8, 0.92, -35, (120, 90))


def couch():
    return (
        R(40, 60, 160, 58, "plum", 18)
        + R(26, 92, 36, 58, "plumhi", 14) + R(178, 92, 36, 58, "plumhi", 14)
        + R(56, 104, 128, 40, "plumhi", 10)
        + L("M120 106 L120 142", "ink", 1.3)
        + scope(R(62, 78, 32, 28, "gold", 8), rot=-10, about=(78, 92))
        + R(44, 148, 8, 10, "dark") + R(188, 148, 8, 10, "dark")
    )


def mixer():
    return (
        R(146, 64, 30, 86, "teal", 12)
        + R(60, 40, 122, 42, "teal", 21)
        + C(164, 60, 6, "cream")
        + R(104, 80, 7, 26, "steel")
        + S("M72 104 L144 104 L136 142 Q108 156 80 142 Z", "steel")
        + R(70, 100, 76, 10, "steeldk", 4)
        + R(58, 146, 128, 12, "sagedk", 6)
    )


def coffee():
    return (
        L("M112 30 Q106 22 112 14 Q118 6 112 -2 M128 30 Q122 22 128 14 Q134 6 128 -2", "steel", 3)
        + R(80, 44, 80, 112, "dark", 10)
        + R(72, 36, 96, 26, "darkhi", 9)
        + R(88, 72, 64, 76, "ink", 6)
        + S("M98 96 L138 96 L142 142 Q120 150 98 142 Z", "glass")
        + S("M98 116 L140 116 L141 140 Q120 148 99 140 Z", "wood")
        + L("M140 104 Q156 106 154 120 Q152 132 141 132", "glass", 5)
        + C(150, 49, 4, "coral")
    )


def fridge():
    return (
        R(78, 28, 92, 124, "glass", 14)
        + L("M78 68 L170 68", "ink", 1.3)
        + R(152, 38, 7, 22, "dark", 3) + R(152, 80, 7, 36, "dark", 3)
        + C(100, 92, 7, "coral") + scope(R(108, 106, 16, 16, "gold", 3), rot=12, about=(116, 114)) + C(98, 130, 5, "plum")
        + R(88, 150, 12, 8, "dark", 2) + R(148, 150, 12, 8, "dark", 2)
    )


def console():
    return (
        S("M88 28 Q112 18 124 34 L130 146 Q114 158 86 150 Z", "cream")
        + S("M102 28 L111 27 L116 150 L105 152 Z", "dark")
        + R(82, 148, 54, 9, "dark", 4)
        + S("M146 118 Q152 106 172 108 Q192 106 198 118 L206 140 Q208 152 198 152 Q190 150 186 140 L158 140 Q154 150 146 152 Q136 152 138 140 Z", "plum")
        + C(160, 124, 5, "cream") + C(186, 119, 3.5, "gold") + C(193, 127, 3.5, "coral") + C(179, 127, 3.5, "teal") + C(186, 134, 3.5, "pink")
    )


def lamp():
    return (
        C(120, 82, 46, "butter", "draw=none,opacity=0.6")
        + R(117, 70, 6, 82, "dark")
        + S("M90 32 L150 32 L166 78 L74 78 Z", "gold")
        + S("M90 32 L150 32 L152 40 L88 40 Z", "golddk")
        + E(120, 152, 30, 7, "dark")
    )


def chair():
    return (
        R(84, 28, 72, 70, "coral", 20)
        + R(98, 40, 44, 10, "coraldk", 5)
        + R(116, 96, 8, 12, "dark")
        + L("M62 88 L80 88 M160 88 L178 88", "dark", 6) + L("M70 88 L70 104 M170 88 L170 104", "dark", 5)
        + R(70, 102, 100, 18, "dark", 9)
        + R(116, 120, 8, 20, "steel")
        + L("M120 140 L84 150 M120 140 L156 150 M120 140 L120 154", "dark", 6)
        + C(84, 154, 5, "dark") + C(156, 154, 5, "dark") + C(120, 157, 5, "dark")
    )


def microwave():
    return (
        R(38, 52, 164, 98, "steel", 14)
        + R(50, 64, 104, 74, "dark", 8)
        + E(102, 122, 34, 7, "darkhi")
        + R(88, 104, 28, 16, "orange", 6)
        + R(164, 66, 26, 12, "teal", 3)
        + "\\foreach \\cx in {171,183} \\foreach \\cy in {94,106} \\filldraw[inked,fill=dark] (\\cx,\\cy) circle [radius=4];\n"
        + R(166, 120, 22, 10, "coral", 5)
        + R(54, 148, 14, 7, "dark") + R(172, 148, 14, 7, "dark")
    )


def dining():
    chair_l = R(28, 56, 10, 100, "coral", 4) + R(28, 104, 40, 10, "coral", 4) + R(58, 110, 8, 46, "coral")
    chair_r = R(202, 56, 10, 100, "coral", 4) + R(172, 104, 40, 10, "coral", 4) + R(174, 110, 8, 46, "coral")
    return (
        chair_l + chair_r
        + R(54, 80, 132, 13, "wood", 4)
        + R(64, 92, 10, 64, "wooddk") + R(166, 92, 10, 64, "wooddk")
        + R(112, 58, 16, 22, "plum", 5) + C(114, 52, 7, "gold") + C(127, 50, 6, "pink")
    )


def kayak():
    return (
        S("M14 116 Q110 76 226 96 Q120 136 14 116 Z", "orange")
        + S("M14 116 Q120 122 226 96 Q120 136 14 116 Z", "coraldk")
        + E(120, 103, 24, 7, "dark", -6)
        + L("M58 58 L182 152", "dark", 5)
        + E(54, 55, 18, 9, "gold", -37) + E(186, 155, 18, 9, "gold", -37)
    )


def mower():
    return (
        L("M66 120 L40 44", "dark", 6) + L("M28 46 L52 40", "dark", 7)
        + S("M48 136 Q48 116 70 116 L168 116 Q186 116 188 132 L188 140 L48 140 Z", "sage")
        + R(92, 90, 64, 32, "coral", 14)
        + R(112, 80, 22, 14, "dark", 4)
        + C(70, 146, 13, "dark") + C(70, 146, 5, "steel")
        + C(170, 146, 13, "dark") + C(170, 146, 5, "steel")
    )


def acoustic():
    body = R(113, 8, 14, 92, "wooddk") + R(109, 0, 22, 20, "dark", 5) + guitar_body("gold", True)
    return scope(body, 8, 8, 0.92, 30, (120, 90))


def turntable():
    return (
        R(36, 100, 168, 42, "wood", 4)
        + S("M50 72 L190 72 L204 100 L36 100 Z", "tan")
        + E(114, 86, 52, 11, "dark")
        + E(114, 86, 30, 6, "darkhi", 0, "draw=none")
        + E(114, 86, 14, 3.5, "coral")
        + L("M184 78 L184 84 L148 92", "steeldk", 4)
        + C(184, 78, 5, "steel")
        + C(60, 121, 7, "dark") + C(84, 121, 7, "dark")
        + R(148, 114, 44, 14, "wooddk", 3)
        + R(42, 140, 12, 8, "dark") + R(186, 140, 12, 8, "dark")
    )


def skateboard():
    body = (
        R(56, 112, 28, 7, "steel", 3) + R(156, 112, 28, 7, "steel", 3)
        + C(60, 126, 9, "gold") + C(80, 126, 9, "gold") + C(160, 126, 9, "gold") + C(180, 126, 9, "gold")
        + R(28, 92, 184, 20, "plum", 10)
        + "\\foreach \\gx in {64,88,...,184} \\fill[gold] (\\gx,102) circle [radius=2.6];\n"
    )
    return scope(body, 0, 6, 1, -10, (120, 104))


def stroller():
    return (
        L("M76 150 L120 128 L156 150", "dark", 5)
        + L("M168 82 L196 44", "dark", 6)
        + S("M54 80 Q54 130 104 130 L150 130 Q170 130 170 110 L170 80 Z", "pink")
        + S("M54 80 Q56 40 120 38 L120 80 Z", "plum")
        + L("M70 80 Q74 56 120 50", "plumhi", 2)
        + C(76, 150, 14, "dark") + C(76, 150, 5, "steel")
        + C(156, 150, 14, "dark") + C(156, 150, 5, "steel")
    )


def airfryer():
    return (
        R(74, 40, 96, 116, "dark", 30)
        + R(84, 100, 76, 46, "darkhi", 10)
        + R(108, 116, 28, 10, "steel", 4)
        + C(122, 70, 13, "teal") + C(122, 70, 5, "mint")
    )


def tv():
    return (
        R(28, 32, 184, 110, "dark", 8)
        + R(37, 41, 166, 90, "glass", 3)
        + C(166, 68, 14, "gold")
        + S("M37 131 L82 84 L112 112 L136 92 L203 131 Z", "sage")
        + R(112, 142, 16, 10, "dark") + R(84, 150, 72, 8, "dark", 4)
    )


def bookshelf():
    books = [
        (78, 40, 10, 28, "coral"), (90, 44, 8, 24, "plum"), (100, 38, 12, 30, "teal"), (114, 46, 8, 22, "gold"),
        (82, 102, 40, 8, "plum"), (86, 94, 34, 8, "gold"), (130, 82, 10, 28, "pink"), (142, 86, 8, 24, "dark"), (152, 80, 10, 30, "coral"),
        (128, 122, 10, 28, "teal"), (140, 126, 8, 24, "plum"), (150, 124, 10, 26, "gold"),
    ]
    return (
        R(66, 24, 108, 134, "wood", 6)
        + "\\foreach \\sy in {32,74,116} \\filldraw[inked,fill=cream] (74,\\sy) rectangle (166,\\sy+36);\n"
        + "".join(R(x, y, w, h, c, 1.5) for x, y, w, h, c in books)
        + C(150, 58, 9, "sage") + R(144, 60, 12, 8, "orange", 2)
        + R(82, 128, 36, 22, "orange", 3)
    )


def plant():
    return (
        S("M120 112 C100 80 60 70 46 92 C58 112 96 118 120 112 Z", "sage")
        + S("M120 112 C140 76 186 70 196 94 C182 114 144 118 120 112 Z", "sagedk")
        + S("M120 112 C106 70 110 32 130 20 C148 46 138 88 120 112 Z", "greenC")
        + L("M70 92 L104 106 M168 90 L138 106 M128 34 L122 96", "forest", 2)
        + S("M88 112 L152 112 L142 156 L98 156 Z", "coral")
        + R(84, 106, 72, 13, "coraldk", 4)
    )


def sneakers():
    return (
        S("M38 132 Q38 98 64 96 L96 98 Q112 80 136 84 L180 108 Q206 114 206 132 L206 138 L38 138 Z", "cream")
        + L("M70 128 Q110 96 170 114", "plum", 9)
        + R(32, 134, 180, 15, "coral", 7)
        + L("M106 94 L116 102 M116 90 L126 98 M126 88 L136 96", "ink", 3)
        + R(40, 104, 18, 12, "gold", 4)
    )


def camera():
    return (
        R(86, 42, 56, 30, "dark", 6)
        + R(42, 58, 156, 92, "dark", 16)
        + R(162, 58, 36, 92, "coral", 14)
        + C(112, 106, 37, "darkhi") + C(112, 106, 24, "glass") + C(112, 106, 11, "dark")
        + C(104, 98, 4, "cream", "draw=none")
        + R(56, 68, 20, 9, "steel", 3)
        + C(178, 50, 7, "steel")
    )


def treadmill():
    return (
        L("M164 128 L182 50", "dark", 7)
        + L("M150 76 L186 76", "dark", 6)
        + scope(R(164, 36, 46, 22, "teal", 6), rot=-12, about=(187, 47))
        + S("M24 134 L178 126 L186 140 L32 148 Z", "dark")
        + L("M36 138 L176 131", "steel", 3, "dash pattern=on 10pt off 8pt")
        + C(40, 146, 6, "coral")
    )


def motorcycle():
    return (
        C(60, 128, 29, "dark") + C(60, 128, 12, "steel")
        + C(184, 128, 29, "dark") + C(184, 128, 12, "steel")
        + L("M184 128 L160 68", "steel", 7)
        + L("M148 62 L172 62", "dark", 6)
        + L("M60 128 L98 110", "dark", 6)
        + R(90, 104, 46, 28, "steel", 6)
        + L("M102 136 L42 140", "steeldk", 6)
        + S("M90 100 Q104 74 138 78 Q158 82 160 102 Z", "orange")
        + S("M50 100 Q68 90 96 96 L96 106 L54 108 Z", "dark")
        + C(166, 84, 8, "gold")
    )


def desk():
    return (
        R(84, 44, 50, 34, "dark", 3) + R(88, 48, 42, 26, "glass", 2) + R(74, 76, 70, 6, "steel", 2)
        + R(156, 62, 14, 16, "coral", 3)
        + R(38, 80, 164, 12, "gold", 4)
        + R(48, 92, 9, 64, "dark")
        + R(144, 92, 48, 64, "golddk", 4)
        + R(150, 102, 36, 20, "gold", 3) + R(150, 128, 36, 20, "gold", 3)
        + R(162, 110, 12, 4, "dark", 2) + R(162, 136, 12, 4, "dark", 2)
    )


def dumbbell(x, y, plate):
    return (
        R(x, y - 4, 96, 8, "steel", 4)
        + R(x + 4, y - 24, 16, 48, "dark", 5) + R(x + 18, y - 17, 11, 34, plate, 4)
        + R(x + 76, y - 24, 16, 48, "dark", 5) + R(x + 67, y - 17, 11, 34, plate, 4)
    )


def dumbbells():
    return dumbbell(40, 98, "plum") + dumbbell(100, 132, "coral")


def scooter():
    return (
        C(66, 140, 15, "dark") + C(66, 140, 6, "steel")
        + C(184, 140, 15, "dark") + C(184, 140, 6, "steel")
        + R(56, 124, 120, 11, "dark", 5)
        + L("M184 140 L174 120 L186 44", "teal", 8)
        + L("M168 42 L204 46", "dark", 7)
        + R(72, 119, 22, 6, "teal", 2)
    )


ITEMS = {
    "toaster": toaster, "jersey": jersey, "sedan": sedan, "bike": bike, "guitar": guitar, "couch": couch,
    "mixer": mixer, "coffee": coffee, "fridge": fridge, "console": console, "lamp": lamp, "chair": chair,
    "microwave": microwave, "dining": dining, "kayak": kayak, "mower": mower, "acoustic": acoustic,
    "turntable": turntable, "skateboard": skateboard, "stroller": stroller, "airfryer": airfryer, "tv": tv,
    "bookshelf": bookshelf, "plant": plant, "sneakers": sneakers, "camera": camera, "treadmill": treadmill,
    "motorcycle": motorcycle, "desk": desk, "dumbbells": dumbbells, "scooter": scooter,
}

# Wall, floor and blob for each listing photo, chosen per item by its seed.
BACKDROPS = [
    ("butter", "tan", "sky"), ("salmon", "wood", "wallD"), ("mint", "tan", "greenC"),
    ("rosewall", "wood", "pink"), ("lilac", "tan", "plumhi"), ("sky", "wood", "skyhi"),
]
BLOBS = [
    "M120 22 C170 18 210 48 210 92 C210 140 168 162 118 160 C68 158 30 134 32 90 C34 46 72 26 120 22 Z",
    "M64 34 C102 10 178 22 204 64 C228 106 194 158 134 160 C82 162 28 148 28 100 C28 68 40 48 64 34 Z",
    "M130 18 C184 26 218 70 204 116 C190 162 122 168 78 152 C34 138 18 96 38 60 C56 28 94 12 130 18 Z",
]


def confetti(rng):
    """Memphis confetti, kept to the four corners so the item stays the subject."""
    out = ""
    corners = [(14, 14), (190, 14), (14, 112), (196, 112)]
    rng.shuffle(corners)
    fills = ["coral", "plum", "gold", "teal", "pink", "sage"]
    for k, (x, y) in enumerate(corners):
        c = rng.choice(fills)
        x += rng.uniform(0, 18)
        y += rng.uniform(0, 16)
        if k == 0:
            out += C(x + 8, y + 8, 7, c)
        elif k == 1:
            out += L(f"M{f(x)} {f(y + 10)} L{f(x + 8)} {f(y)} L{f(x + 16)} {f(y + 10)} L{f(x + 24)} {f(y)}", "ink", 2.4)
        elif k == 2:
            out += S(f"M{f(x)} {f(y + 18)} L{f(x + 10)} {f(y)} L{f(x + 20)} {f(y + 18)} Z", c)
        else:
            out += f"\\foreach \\dx in {{0,9,18}} \\fill[{c}] ({f(x)}+\\dx,{f(y + 6)}) circle [radius=2.6];\n"
    return out


def listing_photo(item):
    rng = random.Random("item:" + item)
    wall, floor, blob = rng.choice(BACKDROPS)
    W, H = SIZES["item"]
    s = "% background: wall, floor, blob, confetti\n"
    s += f"\\fill[{wall}] (0,0) rectangle {P(W, H)};\n"
    s += f"\\filldraw[{style(blob)}] {path(rng.choice(BLOBS))};\n"
    s += confetti(rng)
    s += f"\\fill[{floor}] {P(0, FLOOR)} rectangle {P(W, H)};\n"
    s += L(f"M0 {FLOOR} L{W} {FLOOR}", "ink", 1.3)
    s += f"\\fill[ink, opacity=0.18] {P(120, 160)} ellipse [x radius=78, y radius=6];\n"
    s += "% the item\n" + ITEMS[item]()
    return s, SIZES["item"]


# ---- the street ---------------------------------------------------------------

def cloud(x, y, s):
    puffs = [(-1.0, 0, 0.55), (-0.45, -0.25, 0.7), (0.2, -0.35, 0.8), (0.85, -0.15, 0.6), (1.35, 0.05, 0.45)]
    out = "".join(C(x + dx * s, y + dy * s, r * s, "cloud") for dx, dy, r in puffs)
    out += f"\\fill[cloud] {P(x - s, y - 0.05 * s)} rectangle {P(x + 1.35 * s, y + 0.15 * s)};\n"
    out += L(f"M{f(x - 1.25 * s)} {f(y + 0.15 * s)} L{f(x + 1.6 * s)} {f(y + 0.15 * s)}")
    return out


def building(rng, x, top, w, base, wall):
    s = R(x, top, w, base - top, wall)
    s += R(x - 3, top - 6, w + 6, 8, wall)
    cols = max(1, int((w - 8) // 14))
    rows = max(1, int((base - top - 16) // 18))
    gap = (w - cols * 8) / (cols + 1)
    s += (f"\\foreach \\i in {{0,...,{cols - 1}}} \\foreach \\j in {{0,...,{rows - 1}}} "
          f"\\filldraw[inked, line width=0.8pt, fill=window] ({f(x + gap)}+\\i*{f(gap + 8)},{top + 12}+\\j*18) rectangle ++(8,10);\n")
    for _ in range(cols * rows // 4):
        i, j = rng.randrange(cols), rng.randrange(rows)
        s += f"\\filldraw[inked, line width=0.8pt, fill=windark] {P(x + gap + i * (gap + 8), top + 12 + j * 18)} rectangle ++(8,10);\n"
    kind = rng.choice(["ac", "antenna", "tank", "none"])
    if kind == "ac":
        s += R(x + w * 0.2, top - 18, 18, 12, "metal") + C(x + w * 0.2 + 9, top - 12, 4, "steeldk")
    elif kind == "antenna":
        a = x + w * 0.6
        s += L(f"M{f(a)} {top - 6} L{f(a)} {top - 34} M{f(a - 8)} {top - 26} L{f(a + 8)} {top - 26} M{f(a - 6)} {top - 18} L{f(a + 6)} {top - 18}", "ink", 1.3)
    elif kind == "tank":
        t = x + w * 0.35
        s += L(f"M{f(t + 2)} {top - 6} L{f(t + 2)} {top - 14} M{f(t + 16)} {top - 6} L{f(t + 16)} {top - 14}", "ink", 1.3)
        s += R(t, top - 30, 18, 16, "wood") + S(f"M{f(t - 2)} {top - 30} L{f(t + 9)} {top - 40} L{f(t + 20)} {top - 30} Z", "wooddk")
    return s


def street(rng, W, H, horizon):
    s = "% sky and clouds\n"
    s += f"\\fill[sky] (0,0) rectangle {P(W, H)};\n"
    s += f"\\fill[skyhi] (0,0) rectangle {P(W, 52)};\n"
    s += cloud(70, 40, 26) + cloud(250, 30, 20) + cloud(400, 48, 24)
    s += "% buildings\n"
    x = -10
    walls = ["wallA", "wallB", "wallC", "wallD", "wallE", "wallF", "greenA", "greenC"]
    while x < W:
        w = rng.randint(44, 74)
        top = rng.randint(70, 128)
        s += building(rng, x, top, w, horizon, rng.choice(walls))
        x += w + rng.randint(-4, 6)
    s += "% pavement\n"
    s += f"\\fill[paving] {P(0, horizon)} rectangle {P(W, H)};\n"
    s += L(f"M0 {horizon} L{W} {horizon}")
    s += f"\\fill[ground] {P(0, H - 26)} rectangle {P(W, H)};\n"
    s += L(f"M0 {H - 26} L{W} {H - 26}")
    s += f"\\foreach \\dx in {{20,90,...,480}} \\fill[tan] (\\dx,{H - 14}) rectangle ++(36,4);\n"
    return s


def bunting(x0, y0, x1, y1, sag, n):
    out = L(f"M{x0} {y0} Q{(x0 + x1) / 2} {y0 + sag * 2} {x1} {y1}", "ink", 1.3)
    fills = ["coral", "plum", "gold", "teal", "pink", "sage"]
    for k in range(n):
        t = (k + 0.5) / n
        bx = (1 - t) ** 2 * x0 + 2 * (1 - t) * t * (x0 + x1) / 2 + t * t * x1
        by = (1 - t) ** 2 * y0 + 2 * (1 - t) * t * (y0 + sag * 2) + t * t * y1
        out += S(f"M{f(bx - 7)} {f(by)} L{f(bx + 7)} {f(by)} L{f(bx)} {f(by + 15)} Z", fills[k % len(fills)])
    return out


def tag(x, y, s, fill, rot=0):
    body = S("M0 0 L26 0 L36 10 L26 20 L0 20 Z", fill) + C(28, 10, 3, "cream")
    return scope(body, x, y, s, rot, (0, 10))


def item_at(name, x, y, s):
    """An item drawing placed with its 240 x 180 box's top-left at (x, y),
    scaled, its outlines thinned to match (never below a hairline that reads)."""
    k = max(s, 0.45)
    LINE.append(k)
    SIZE.append(s)
    body = ITEMS[name]()
    SIZE.pop()
    LINE.pop()
    return scope(body, x, y, s, ink=1.3 * k)


def phone(x, y):
    """A phone showing a listing: a photo, a price tag and grey lines where words go."""
    s = R(x, y, 96, 176, "dark", 16)
    s += R(x + 7, y + 14, 82, 150, "cream", 8)
    s += R(x + 13, y + 22, 70, 52, "butter", 4)
    s += scope(jersey(), x + 13, y + 22, 0.29)
    s += tag(x + 15, y + 82, 0.9, "coral")
    s += "".join(R(x + 15, y + yy, ww, 6, "steel", 3, "draw=none") for yy, ww in [(106, 64), (118, 54), (130, 60)])
    s += R(x + 15, y + 142, 30, 14, "plum", 5) + R(x + 50, y + 142, 30, 14, "sage", 5)
    s += R(x + 38, y + 6, 20, 4, "darkhi", 2, "draw=none")
    return s


def table(x, y, w):
    return R(x, y, w, 9, "wood", 3) + R(x + 8, y + 9, 7, 40, "wooddk") + R(x + w - 15, y + 9, 7, 40, "wooddk") + R(x - 4, y - 2, w + 8, 4, "cream", 2)


def title():
    rng = random.Random("title")
    W, H = SIZES["scene"]
    s = street(rng, W, H, 196)
    s += "% bunting over the yard sale\n" + bunting(150, 120, 470, 112, 16, 11)
    s += "% the tables and what is on them\n"
    s += table(168, 210, 136) + table(322, 210, 140)
    s += item_at("toaster", 160, 164, 0.32) + item_at("plant", 216, 132, 0.5)
    s += item_at("lamp", 306, 124, 0.56) + item_at("console", 372, 160, 0.34) + item_at("turntable", 396, 172, 0.27)
    s += tag(186, 222, 0.7, "gold", 70) + tag(268, 222, 0.7, "teal", 80) + tag(352, 222, 0.7, "pink", 75) + tag(432, 222, 0.7, "coral", 85)
    s += "% the phone with a listing on it\n" + phone(28, 60)
    return s, (W, H)


def final():
    rng = random.Random("final")
    W, H = SIZES["scene"]
    s = street(rng, W, H, 196)
    s += "% the car home, loaded: what pokes out of the windows goes in first\n"
    s += item_at("lamp", 164, 124, 0.3) + item_at("plant", 230, 132, 0.28)
    s += item_at("sedan", 130, 88, 0.9)
    s += item_at("couch", 184, 84, 0.45)
    s += L("M206 152 L206 160 M270 152 L270 160", "dark", 2.2)
    s += "% speed lines and dust\n"
    s += L("M60 186 L118 186 M44 202 L110 202 M70 218 L124 218", "ink", 2.2)
    s += C(170, 226, 9, "cream") + C(154, 222, 6, "cream") + C(142, 228, 4, "cream")
    s += bunting(10, 150, 120, 140, 10, 5) + tag(400, 210, 1.1, "coral", -20) + tag(424, 150, 0.9, "gold", 18)
    return s, (W, H)


def icon():
    body = S("M10 18 L36 8 L58 30 L32 56 L10 34 Z", "coral") + C(24, 24, 5, "cream")
    return body, SIZES["icon"]


PICTURES = {"title": title, "final": final, "icon": icon}
for _k in ITEMS:
    PICTURES[f"items/{_k}"] = (lambda k=_k: listing_photo(k))

DPI = {"items": 150, "": 170}
DPI_OF = {"icon": 384}


def document(body, size):
    W, H = size
    return PREAMBLE + "\\begin{document}\n\\begin{tikzpicture}\n" + f"\\clip (0,0) rectangle {P(W, H)};\n" + body + "\\end{tikzpicture}\n\\end{document}\n"


def main():
    only = set(sys.argv[1:])
    for name, make in PICTURES.items():
        if only and name not in only and name.split("/")[0] not in only:
            continue
        body, size = make()
        tex = document(body, size)
        folder = name.split("/")[0] if "/" in name else ""
        dpi = DPI_OF.get(name, DPI.get(folder, 150))
        with tempfile.TemporaryDirectory() as tmp:
            t = Path(tmp)
            (t / "art.tex").write_text(tex)
            r = subprocess.run(["pdflatex", "-interaction=nonstopmode", "-halt-on-error", "art.tex"], cwd=t, capture_output=True, text=True)
            if r.returncode != 0:
                print(r.stdout[-3000:])
                raise SystemExit(f"{name}: pdflatex failed")
            clear = ["-transp"] if name == "icon" else []
            subprocess.run(["pdftocairo", "-png", *clear, "-r", str(dpi), "-singlefile", str(t / "art.pdf"), str(t / "art")], check=True)
            out = OUT / f"{name}.png"
            out.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(t / "art.png", out)
            src = SRC / f"{name}.tex"
            src.parent.mkdir(parents=True, exist_ok=True)
            src.write_text(tex)
        print("wrote", out)


if __name__ == "__main__":
    main()
