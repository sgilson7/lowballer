# Plan

Stage 2, from `BRIEF.md`. The brief approves the plan in advance, so the build followed it without a stop at stage 3.

| # | Decision | Reason | Test that shows it holds |
| --- | --- | --- | --- |
| 1 | One hidden number per seller, `flex` (0 to 1000), drives every clue: temperament bucket, tone tier, voice, profile, reply, and the minimum. | The brief: a seller's tendency is "consistently reflected" in description, profile and messages. | `a_seller_writes_in_one_voice_everywhere`, `flexible_sellers_sell_for_less_on_their_profiles` |
| 2 | Tone tier = flex × 3 plus up to ±0.6 of a tier of drift. A stubborn seller never says "offers accepted"; a flexible one never says "firm". | "Correspond somewhat strongly with the tone": strong, but not a proof. | `tone_follows_flex_within_the_drift`, `the_tone_phrase_appears_in_the_description` |
| 3 | Minimum = asking × (1 − discount), discount = 2% + 3.5% per tone tier + up to 20% from flex + up to 6% for age + three noise draws of ±2%, clamped to 0 to 45%, in basis points. | A function of price, description, attitude and randomness, as the brief asks; integer maths (Builder D2). | `the_minimum_matches_a_hand_worked_case`, `firm_listings_hold_closer_to_asking_than_offers_accepted` |
| 4 | Asking prices: multiples of 5 under $100, of 25 from $100; real-world items from a $15 toaster to a $4,500 car. Minimums are to the dollar. | The brief's pricing rules and examples. | `prices_are_round_and_minimums_sit_at_or_below_asking`; `data::parse` refuses a non-round price |
| 5 | A message gets a reply in the seller's voice. Flexible and steady sellers name a counter between the minimum and asking; half of stubborn sellers refuse to name one. A counter is always one the seller would accept. | Messages must be informative but never a lie. | `a_counter_offer_in_a_message_is_always_one_the_seller_would_accept` |
| 6 | A profile shows joined year, rating, items sold, response time, bio, two reviews (one in five from a neighbouring temperament) and two recent sales whose discount follows flex. | Profile clues point the right way most of the time. | `flexible_sellers_sell_for_less_on_their_profiles` |
| 7 | Two peeks, two messages, five offers per screen, never peek and message one listing; an offer is final. The fifth offer ends the screen. | The brief's limits. | `two_peeks_and_two_messages_per_screen_never_both_on_one_listing`, `the_fifth_offer_ends_the_screen_and_scores_it_by_hand` |
| 8 | Score: accepted saves asking − offer; declined saves $0. Final: saved ÷ asking of the fifteen items bid on, to a tenth of a percent, with a rank. | "A percentage of money spent, i.e. they on average lowballed 23%." | `the_final_tally_is_saved_over_asking_of_items_bid_on` |
| 9 | Results show accepted or declined for each offer and the minimum of the skipped listing only. | The brief names only the sixth item's minimum. | `the_view_never_carries_a_hidden_number_before_its_time`; browser check |
| 10 | Art: TikZ from `tools/art.py`, one 4:3 photo per item, a title and a final scene, in Sam_Comm_final's palette with Corporate Memphis blobs and confetti. | Sam's instruction; the house art pipeline. | `tests/art.py` (every item has its picture) |

## Open questions for the person

1. Is a 0 to 45% range for the minimum's discount the right spread? Measured over 18,000 listings: a flat offer of 90% of asking is accepted 86% of the time, 85% → 71%, 80% → 53%, 75% → 38%, 70% → 21%. So reading the clues is what lifts a score.
2. Should the results also reveal the minimums of declined offers? The brief names only the skipped item's; it would teach players faster.

## Status

- [x] Approved in advance by the brief (Autonomy).
