//! Behaviour tests over the real content in `data/`. Expected values are
//! worked out by hand from the brief, not copied from the game's output.

use std::fs;
use std::path::Path;

use lowball_core::api;
use lowball_core::data::{self, Data};
use lowball_core::game::{self, Action, Game, Outcome, Phase, LISTINGS, OFFERS, ROUNDS};
use lowball_core::market::{price_is_round, Bucket};

fn content() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let read = |f: &str| fs::read_to_string(root.join(f)).expect("read data file");
    format!(
        "{{\"items\":{},\"sellers\":{},\"copy\":{}}}",
        read("items.json"),
        read("sellers.json"),
        read("copy.json")
    )
}

fn data() -> Data {
    data::parse(&content()).expect("the content files load")
}

fn browsing(d: &Data, seed: u64) -> Game {
    game::apply(&game::new_game(d, seed), &Action::Start).unwrap()
}

fn offer(g: &Game, i: usize, amount: &str) -> Result<Game, &'static str> {
    game::apply(
        g,
        &Action::Offer {
            listing: i.to_string(),
            amount: amount.into(),
        },
    )
}

fn peek(g: &Game, i: usize) -> Result<Game, &'static str> {
    game::apply(
        g,
        &Action::Peek {
            listing: i.to_string(),
        },
    )
}

fn message(g: &Game, i: usize) -> Result<Game, &'static str> {
    game::apply(
        g,
        &Action::Message {
            listing: i.to_string(),
        },
    )
}

#[test]
fn a_game_has_three_screens_of_six_different_items_and_sellers() {
    let d = data();
    let g = game::new_game(&d, 1);
    assert_eq!(g.phase, Phase::Title);
    assert_eq!(g.rounds.len(), ROUNDS);
    let mut items = Vec::new();
    let mut names = Vec::new();
    for r in &g.rounds {
        assert_eq!(r.listings.len(), LISTINGS);
        for l in &r.listings {
            items.push(l.item.clone());
            names.push(l.seller.name.clone());
        }
    }
    items.sort();
    items.dedup();
    names.sort();
    names.dedup();
    assert_eq!(items.len(), ROUNDS * LISTINGS);
    assert_eq!(names.len(), ROUNDS * LISTINGS);
}

#[test]
fn the_same_seed_deals_the_same_game() {
    let d = data();
    assert_eq!(game::new_game(&d, 99), game::new_game(&d, 99));
    assert_ne!(game::new_game(&d, 99), game::new_game(&d, 100));
}

#[test]
fn prices_are_round_and_minimums_sit_at_or_below_asking() {
    let d = data();
    for seed in 0..200 {
        for r in &game::new_game(&d, seed).rounds {
            for l in &r.listings {
                let item = d.items.iter().find(|i| i.id == l.item).unwrap();
                assert!(item.prices.contains(&l.list_price));
                assert!(price_is_round(l.list_price));
                assert!(l.min_price >= 1 && l.min_price <= l.list_price);
            }
        }
    }
}

/// Mean discount (basis points) of the minimum from asking, for each tone tier.
fn mean_discount_by_tone(d: &Data) -> [i64; 4] {
    let mut sum = [0i64; 4];
    let mut count = [0i64; 4];
    for seed in 0..600 {
        for r in &game::new_game(d, seed).rounds {
            for l in &r.listings {
                let t = l.tone as usize;
                sum[t] += (l.list_price - l.min_price) as i64 * 10_000 / l.list_price as i64;
                count[t] += 1;
            }
        }
    }
    let mut mean = [0; 4];
    for t in 0..4 {
        assert!(count[t] > 100, "tone {t} too rare to measure: {}", count[t]);
        mean[t] = sum[t] / count[t];
    }
    mean
}

#[test]
fn firm_listings_hold_closer_to_asking_than_offers_accepted() {
    let m = mean_discount_by_tone(&data());
    // Each tier is looser than the one before, and "firm" to "offers accepted"
    // is at least ten points of discount apart.
    assert!(m[0] < m[1] && m[1] < m[2] && m[2] < m[3], "{m:?}");
    assert!(m[3] - m[0] >= 1000, "{m:?}");
}

#[test]
fn the_tone_phrase_appears_in_the_description() {
    let d = data();
    for seed in 0..50 {
        for r in &game::new_game(&d, seed).rounds {
            for l in &r.listings {
                let phrases = &d.sellers.tone_phrases[l.tone as usize];
                let desc = l.description.to_lowercase();
                assert!(
                    phrases.iter().any(|p| desc.contains(&p.to_lowercase())),
                    "tone {} phrase missing from {:?}",
                    l.tone,
                    l.description
                );
            }
        }
    }
}

#[test]
fn a_seller_writes_in_one_voice_everywhere() {
    let d = data();
    let lower = |pool: &[String]| pool.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>();
    for seed in 0..50 {
        for r in &game::new_game(&d, seed).rounds {
            for l in &r.listings {
                let p = &l.seller.profile;
                let bios = lower(d.sellers.bios.of(l.bucket));
                assert!(
                    bios.contains(&p.bio.to_lowercase()),
                    "bio from another temperament"
                );
                let closers = lower(d.sellers.closers.of(l.bucket));
                assert!(
                    closers
                        .iter()
                        .any(|c| l.description.to_lowercase().ends_with(c.as_str())),
                    "description closer from another temperament: {}",
                    l.description
                );
                // The first review always matches the seller's temperament.
                assert!(d.sellers.reviews.of(l.bucket).contains(&p.reviews[0].text));
                if l.bucket == Bucket::Flexible && l.description == l.description.to_lowercase() {
                    assert_eq!(
                        l.seller.reply,
                        l.seller.reply.to_lowercase(),
                        "casual everywhere or nowhere"
                    );
                }
            }
        }
    }
}

/// The dollar figures in a line of text: "$1,250" -> 1250.
fn dollars(text: &str) -> Vec<u32> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find('$') {
        let digits: String = rest[at + 1..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == ',')
            .filter(|c| *c != ',')
            .collect();
        if let Ok(n) = digits.parse() {
            out.push(n);
        }
        rest = &rest[at + 1..];
    }
    out
}

#[test]
fn a_counter_offer_in_a_message_is_always_one_the_seller_would_accept() {
    let d = data();
    let mut counters = 0;
    for seed in 0..200 {
        for r in &game::new_game(&d, seed).rounds {
            for l in &r.listings {
                for c in dollars(&l.seller.reply) {
                    counters += 1;
                    assert!(c >= l.min_price && c <= l.list_price, "{c} vs {l:?}");
                }
            }
        }
    }
    assert!(counters > 1000);
}

#[test]
fn flexible_sellers_sell_for_less_on_their_profiles() {
    let d = data();
    let (mut stub, mut flex) = ((0u64, 0u64), (0u64, 0u64));
    for seed in 0..200 {
        for r in &game::new_game(&d, seed).rounds {
            for l in &r.listings {
                for s in &l.seller.profile.sales {
                    let off = (s.listed - s.sold) as u64 * 10_000 / s.listed as u64;
                    match l.bucket {
                        Bucket::Stubborn => stub = (stub.0 + off, stub.1 + 1),
                        Bucket::Flexible => flex = (flex.0 + off, flex.1 + 1),
                        Bucket::Steady => {}
                    }
                }
            }
        }
    }
    assert!(flex.0 / flex.1 > stub.0 / stub.1 + 1000);
}

#[test]
fn two_peeks_and_two_messages_per_screen_never_both_on_one_listing() {
    let d = data();
    let g = browsing(&d, 5);
    let g = peek(&g, 0).unwrap();
    assert_eq!(peek(&g, 0), Err("already-peeked"));
    assert_eq!(message(&g, 0), Err("used-peek-here"));
    let g = peek(&g, 1).unwrap();
    assert_eq!(peek(&g, 2), Err("no-peeks-left"));
    let g = message(&g, 2).unwrap();
    assert_eq!(peek(&g, 2), Err("used-message-here"));
    assert_eq!(message(&g, 2), Err("already-messaged"));
    let g = message(&g, 3).unwrap();
    assert_eq!(message(&g, 4), Err("no-messages-left"));
    assert_eq!(peek(&g, 9), Err("no-such-listing"));
}

#[test]
fn offers_are_checked_before_they_count() {
    let d = data();
    let g = browsing(&d, 6);
    let list = g.current().listings[0].list_price;
    assert_eq!(offer(&g, 0, &(list + 1).to_string()), Err("over-asking"));
    assert_eq!(offer(&g, 0, "eighty"), Err("bad-amount"));
    assert_eq!(offer(&g, 0, "0"), Err("bad-amount"));
    assert_eq!(offer(&g, 0, "12.50"), Err("bad-amount"));
    let g = offer(&g, 0, &format!("${list}")).unwrap();
    assert_eq!(g.current().offers[0], Some(list));
    assert_eq!(offer(&g, 0, "1"), Err("already-offered"));
    assert_eq!(peek(&g, 0), Err("already-offered"));
}

#[test]
fn amounts_parse_as_people_type_them() {
    assert_eq!(game::parse_amount("80"), Some(80));
    assert_eq!(game::parse_amount(" $1,250 "), Some(1250));
    assert_eq!(game::parse_amount("$"), None);
    assert_eq!(game::parse_amount("-5"), None);
    assert_eq!(game::parse_amount("12345678"), None);
}

/// A screen with known minimums: asking $100 each, minimum $77 each.
fn fixed_screen(d: &Data) -> Game {
    let mut g = browsing(d, 7);
    for r in &mut g.rounds {
        for l in &mut r.listings {
            l.list_price = 100;
            l.min_price = 77;
        }
    }
    g
}

#[test]
fn the_fifth_offer_ends_the_screen_and_scores_it_by_hand() {
    let d = data();
    let mut g = fixed_screen(&d);
    // 80 accepted (saves 20), 77 accepted (saves 23), 76 declined (0),
    // 100 accepted (0), 50 declined (0). Listing 5 skipped.
    for (i, a) in ["80", "77", "76", "100", "50"].iter().enumerate() {
        assert_eq!(g.phase, Phase::Browse);
        g = offer(&g, i, a).unwrap();
    }
    assert_eq!(g.phase, Phase::Summary);
    assert_eq!(offer(&g, 5, "90"), Err("not-browsing"));
    let o = game::outcomes(&g, 0);
    assert_eq!(o[0], Outcome::Accepted { offer: 80, saved: 20 });
    assert_eq!(o[1], Outcome::Accepted { offer: 77, saved: 23 });
    assert_eq!(o[2], Outcome::Declined { offer: 76 });
    assert_eq!(o[5], Outcome::Skipped { min: 77 });
    assert_eq!(game::round_totals(&g, 0), (43, 500));
    assert_eq!(game::saved_so_far(&g), 43);
}

#[test]
fn the_final_tally_is_saved_over_asking_of_items_bid_on() {
    let d = data();
    let mut g = fixed_screen(&d);
    for _ in 0..ROUNDS {
        for i in 0..OFFERS {
            g = offer(&g, i, "80").unwrap();
        }
        g = game::apply(&g, &Action::Next).unwrap();
    }
    assert_eq!(g.phase, Phase::Final);
    // 15 accepted offers of $80 on $100: saved $300 of $1,500 asked = 20.0%.
    let t = game::tally(&g);
    assert_eq!(
        (t.saved, t.asking, t.accepted, t.offered, t.pct_tenths),
        (300, 1500, 15, 15, 200)
    );
    assert_eq!(t.rank, 3);
    assert_eq!(game::apply(&g, &Action::Next), Err("bad-action"));
}

#[test]
fn the_view_never_carries_a_hidden_number_before_its_time() {
    let content = content();
    let started: serde_json::Value = serde_json::from_str(&api::start(&content, "3")).unwrap();
    let state = started["ok"]["state"].to_string();
    let browse: serde_json::Value =
        serde_json::from_str(&api::act(&content, &state, r#"{"kind":"start"}"#)).unwrap();
    let view = browse["ok"]["view"].to_string();
    assert!(!view.contains("min_price") && !view.contains("flex"));
    assert_eq!(browse["ok"]["view"]["phase"], "browse");
    assert_eq!(
        browse["ok"]["view"]["browse"]["cards"].as_array().unwrap().len(),
        LISTINGS
    );
    // No card names its seller's minimum as a figure anywhere on it.
    let g: Game = serde_json::from_value(browse["ok"]["state"].clone()).unwrap();
    for (card, l) in browse["ok"]["view"]["browse"]["cards"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&g.current().listings)
    {
        let mut text = card.to_string();
        text = text.replace(&format!("\"{}\"", l.min_price), "");
        if l.min_price != l.list_price {
            assert!(
                !dollars(&text).contains(&l.min_price) || dollars(&l.seller.reply).contains(&l.min_price)
            );
        }
    }
}

#[test]
fn the_api_reports_errors_in_the_players_words() {
    let content = content();
    let started: serde_json::Value = serde_json::from_str(&api::start(&content, "3")).unwrap();
    let state = started["ok"]["state"].to_string();
    let out: serde_json::Value =
        serde_json::from_str(&api::act(&content, &state, r#"{"kind":"next"}"#)).unwrap();
    assert_eq!(out["error"], "That move isn't available right now.");
    let out: serde_json::Value =
        serde_json::from_str(&api::act(&content, "{}", r#"{"kind":"start"}"#)).unwrap();
    assert!(out["error"]
        .as_str()
        .unwrap()
        .starts_with("The game could not be read back"));
}

#[test]
fn the_offer_hint_reads_the_typed_amount() {
    let d = data();
    let g = fixed_screen(&d);
    assert_eq!(lowball_core::view::hint(&d, &g, 0, "80"), "20% below asking");
    assert_eq!(lowball_core::view::hint(&d, &g, 0, "100"), "Full asking price");
    assert_eq!(
        lowball_core::view::hint(&d, &g, 0, "101"),
        "That's more than they're asking."
    );
    assert_eq!(lowball_core::view::offer_start(100), 85);
    assert_eq!(lowball_core::view::offer_start(4000), 3400);
    assert_eq!(lowball_core::view::offer_start(25), 22);
}

#[test]
fn listing_ages_read_in_hours_days_or_weeks() {
    let d = data();
    let a = |h| lowball_core::view::age(&d.copy, h);
    assert_eq!(a(1), "1 hour ago");
    assert_eq!(a(23), "23 hours ago");
    assert_eq!(a(24), "1 day ago");
    assert_eq!(a(335), "13 days ago");
    assert_eq!(a(336), "2 weeks ago");
}
