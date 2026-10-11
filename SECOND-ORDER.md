# Second-order notebook

Stage 6, kept as the work happened. **Kind** is *divergence*, *finding* or *worklist*. **Status** is *done*, *open* or *person*.

| # | Kind | What was assumed or found | What is true | What changed | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | divergence | The brief says "Facebook Marketplace". | Using its name or look would impersonate a real product. | The platform is a made-up one, Curbside (`data/copy.json`). | done |
| 2 | divergence | Prices "in increments of $50–$100 for higher items". | Real listings above $100 often end in 25 ($175). | Round means a multiple of 25 from $100 (`market::price_is_round`); counters and sale prices round to $5 from $100 and $50 from $1,000. | done |
| 3 | finding | The page may not use `Math` or any number but 0 and 1 (Builder D3), yet a game needs a seed. | `crypto.getRandomValues` into a one-element array needs neither. | The page sends the seed as a string; the core parses it. | done |
| 4 | finding | Hand-worked counter in `a_counter_offer_is_never_below...` expected $74 and $87. | $100 items count in $5 steps, so $75 and $90. | Expected values corrected; the rule was right. | done |
| 5 | divergence | `offer_start` documented as "rounded up" but rounded to nearest. | $25 × 85% = $21.25 should start at $22. | Code changed to round up, matching the doc and the hand case. | done |
| 6 | finding | The sheet stayed open after an offer, covering the grid (browser check timed out). | Closing it is the better flow; the card shows "Offered $X". | The page closes the sheet after any accepted send. | done |
| 7 | finding | `display: grid` on the title and final sections overrode the `hidden` attribute: both stayed visible during play. Found on the screenshots, not by the browser check. | A `[hidden] { display: none !important }` rule fixes it. | Added to `style.css`. The browser check asserts text, not visibility of other sections; noted for a future check. | done |
| 8 | finding | TikZ scales coordinates but not lengths in mm or pt, so small items in the scenes had fat outlines and huge corners. | Line widths and corner radii must scale with the placed drawing. | `item_at` pushes the scale; `R` and `L` read it. | done |
| 9 | finding | Break-test pass (rule 6): `python3 scripts/break_tests.py` breaks 14 rules once each. | All 14 fail their tests, e.g. "tone 0 too rare to measure: 0", "casual everywhere or nowhere", "assertion `left == right` failed". | None. | done |
| 10 | worklist | The minimum-price tuning and the open questions in `PLAN.md`. | Sam's or the friend's call after playing. | | person |
