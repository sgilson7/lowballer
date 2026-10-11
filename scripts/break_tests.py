#!/usr/bin/env python3
"""Rule 6 (CLAUDE.md): break each behaviour a test covers, run that test,
expect it to fail, and put the code back. A test that still passes with
its behaviour broken does not cover it.

    python3 scripts/break_tests.py            # every break below
    python3 scripts/break_tests.py vent wall  # the breaks whose names hold these words

Each break is (name, file, text to replace, replacement, test filter). The
file is restored after each run, whatever happens. Exits non-zero if any
break leaves its test passing, or names text no longer in its file.
"""
import pathlib, re, subprocess, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]

BREAKS = [
 ("tone tracks flex", "crates/core/src/market.rs", "let x = flex as i64 * 3 + drift;", "let x = 1500 + drift;", "firm_listings_hold_closer"),
 ("tone moves the minimum", "crates/core/src/market.rs", "BASE_DISCOUNT + TONE_STEP * tone as i64", "BASE_DISCOUNT + TONE_STEP * 0", "the_minimum_matches"),
 ("peek blocked by a message", "crates/core/src/game.rs", "    } else if r.messaged.contains(&i) {\n        Some(\"used-message-here\")", "    } else if false {\n        Some(\"used-message-here\")", "two_peeks_and_two_messages"),
 ("message blocked by a peek", "crates/core/src/game.rs", "    } else if r.peeked.contains(&i) {\n        Some(\"used-peek-here\")", "    } else if false {\n        Some(\"used-peek-here\")", "two_peeks_and_two_messages"),
 ("two peeks", "crates/core/src/game.rs", "pub const PEEKS: usize = 2;", "pub const PEEKS: usize = 3;", "two_peeks_and_two_messages"),
 ("offer at the minimum is accepted", "crates/core/src/game.rs", "Some(o) if o >= listing.min_price", "Some(o) if o > listing.min_price", "the_fifth_offer_ends"),
 ("fifth offer ends the screen", "crates/core/src/game.rs", "if round.offers_made() == OFFERS {", "if round.offers_made() == OFFERS + 1 {", "the_fifth_offer_ends"),
 ("over asking", "crates/core/src/game.rs", "if n > game.current().listings[i].list_price {", "if n > game.current().listings[i].list_price * 2 {", "offers_are_checked"),
 ("tally over items bid on", "crates/core/src/game.rs", "        .filter(|(_, o)| o.is_some())\n", "", "the_final_tally"),
 ("counter at or above the minimum", "crates/core/src/market.rs", "ceil_to(raw, price_step(list)).min(list)", "(raw / price_step(list) * price_step(list)).min(list)", "a_counter_offer_in_a_message"),
 ("one voice everywhere", "crates/core/src/market.rs", "    lower_if(line, casual)", "    line", "a_seller_writes_in_one_voice"),
 ("profile sales follow flex", "crates/core/src/market.rs", "let off = (100 + 3 * flex as i64", "let off = (1600 + 0 * flex as i64", "flexible_sellers_sell_for_less"),
 ("weeks", "crates/core/src/view.rs", "} else if hours < 24 * 14 {", "} else if hours < 24 * 15 {", "listing_ages_read"),
 ("hidden minimum", "crates/core/src/view.rs", "        alt: fill(&copy.photo_alt, &[(\"title\", l.title.clone())]),\n        price: money(list),", "        alt: fill(&copy.photo_alt, &[(\"title\", money(l.min_price))]),\n        price: money(list),", "the_view_never_carries"),
]


def main():
    words = sys.argv[1:]
    bad = 0
    for name, path, old, new, test in BREAKS:
        if words and not any(w in name for w in words):
            continue
        p = ROOT / path
        src = p.read_text()
        if old not in src:
            print(f"{name}: the text to break is gone from {path}")
            bad += 1
            continue
        p.write_text(src.replace(old, new, 1))
        try:
            r = subprocess.run(["cargo", "test", "-q", "-p", "lowball_core", test], cwd=ROOT, capture_output=True, text=True)
            out = r.stdout + r.stderr
            m = re.search(r"panicked at [^\n]*\n([^\n]*)", out)
            msg = m.group(1).strip() if m else ("does not compile" if "error[" in out else "")
            ok = r.returncode != 0
            bad += not ok
            print(f"{name}: {'fails, as it should' if ok else 'STILL PASSES'} ({test}) {msg[:140]}")
        finally:
            p.write_text(src)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
