# Handoff

A fresh session starts from this file, not from memory.

## What changed

The MVP, built from `BRIEF.md` on the Builder template:

- `crates/core`: the market (seller flexibility → tone, voice, profile, reply, hidden minimum), the game (two peeks, two messages, five offers per screen; scoring; final tally and rank), and the view the page draws. 41 tests; `scripts/break_tests.py` breaks 14 rules and each is caught.
- `data/`: 31 items with real-world asking prices, seller voices by temperament, every word on screen.
- `web/`: the page (title, grid, listing sheet, results, final tally) and 34 TikZ pictures from `tools/art.py`.
- Deployed: <https://sgilson7.github.io/lowballer/> from <https://github.com/sgilson7/lowballer> (Pages via `.github/workflows/deploy.yml`, Chromium check in CI).
- Listed on the site: <https://sgilson7.github.io/projects/lowballer/>, with a still and a clip tile (website commit 9ac4927, build f7ac80d1).

## What passed

2026-10-10, before the first push:

```
WASM IMPORT AUDIT PASSED
BROWSER CHECK PASSED in chromium, firefox, webkit
BUILDER CHECK PASSED
```

After deploy: `tests/browser/check.py` against `https://sgilson7.github.io/lowballer` passed in Chromium; the site's `make gate-live` passed.

## What remains

- `PLAN.md` open questions 1 and 2 (the discount spread; whether to reveal declined minimums).
- `SECOND-ORDER.md` row 7: the browser check asserts text, not that other screens are hidden. A visibility assertion would have caught that bug.
- By hand: play a game and check that firm listings feel firmer than "offers accepted" ones, and that the art reads at phone width.

## What the next session must not assume

- The page holds no rule and no number but 0 and 1 (`boundary.rs`). A new on-screen figure is formatted in `crates/core/src/view.rs`.
- The art PNGs are committed; CI does not run `tools/art.py`. Rerun it locally after changing a drawing.
- The site repository (`../website`) has its own rules on copy (its `CLAUDE.md` §0a); the Lowballer page traces to this repo's README and brief.
