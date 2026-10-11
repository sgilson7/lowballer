//! How a listing is made: the seller's temperament, the tone of the
//! description, the asking price, how long it has been up, and the hidden
//! minimum the seller will take.
//!
//! One hidden number drives everything about a seller: `flex`, in thousandths,
//! 0 (will not budge) to 1000 (will take almost anything). It sets
//!
//! - the temperament bucket, which picks the voice of the description, the
//!   profile's bio, reviews and response time, and the reply to a message;
//! - the tone tier (firm, plain, "or best offer", "offers accepted"), with
//!   noise, so the phrase in the description is a strong clue but not a proof;
//! - the prices on the profile's recent sales;
//! - and, with the tone, the listing's age and more noise, the minimum.
//!
//! Money is whole dollars; fractions are basis points (1/100 of a percent) or
//! thousandths, so there are no floats.

use serde::{Deserialize, Serialize};

use crate::data::{Data, Item};
use crate::rng::Rng;
use crate::text::{ceil_to, div_round, fill, money};

/// Tone tiers: 0 firm, 1 plain, 2 "or best offer", 3 "offers accepted".
pub const TONES: usize = 4;

/// The flex values at which a seller stops being stubborn, and becomes flexible.
pub const STEADY_FROM: u32 = 334;
pub const FLEXIBLE_FROM: u32 = 667;

/// The minimum's discount from asking, in basis points:
/// `BASE + TONE_STEP * tone + FLEX_WEIGHT * flex / 1000 + age bonus + noise`,
/// clamped to `0..=MAX_DISCOUNT`. A firm, stubborn, brand-new listing comes out
/// near 2% off; an "offers accepted", flexible, month-old one near 38%.
pub const BASE_DISCOUNT: i64 = 200;
pub const TONE_STEP: i64 = 350;
pub const FLEX_WEIGHT: i64 = 2000;
pub const MAX_DISCOUNT: i64 = 4500;
/// Each of three noise draws is uniform in `-NOISE..=NOISE`.
pub const NOISE: i64 = 200;
/// How far, in thousandths of a tier, the tone can drift from where flex puts it.
pub const TONE_DRIFT: i64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bucket {
    Stubborn,
    Steady,
    Flexible,
}

pub fn bucket_of(flex: u32) -> Bucket {
    if flex < STEADY_FROM {
        Bucket::Stubborn
    } else if flex < FLEXIBLE_FROM {
        Bucket::Steady
    } else {
        Bucket::Flexible
    }
}

/// An asking price a real seller would type: a multiple of 5 under $100, of
/// 25 from $100.
pub fn price_is_round(price: u32) -> bool {
    price > 0
        && if price < 100 {
            price.is_multiple_of(5)
        } else {
            price.is_multiple_of(25)
        }
}

/// The step a counter-offer or a sale price is rounded to.
pub fn price_step(price: u32) -> u32 {
    if price >= 1000 {
        50
    } else if price >= 100 {
        5
    } else {
        1
    }
}

/// Older listings make sellers more willing, in basis points.
pub fn age_bonus(hours: u32) -> i64 {
    match hours {
        0..=5 => 0,
        6..=23 => 100,
        24..=71 => 250,
        72..=167 => 400,
        168..=335 => 500,
        _ => 600,
    }
}

pub fn discount_bps(tone: u8, flex: u32, hours: u32, noise: i64) -> u32 {
    let d =
        BASE_DISCOUNT + TONE_STEP * tone as i64 + FLEX_WEIGHT * flex as i64 / 1000 + age_bonus(hours) + noise;
    d.clamp(0, MAX_DISCOUNT) as u32
}

/// Asking less the discount, rounded to the dollar, never below $1 or above asking.
pub fn minimum_price(list: u32, discount: u32) -> u32 {
    let m = div_round(list as u64 * (10_000 - discount as u64), 10_000) as u32;
    m.clamp(1, list)
}

/// Where flex puts the tone, moved by up to `TONE_DRIFT` thousandths of a tier.
pub fn tone_of(flex: u32, drift: i64) -> u8 {
    let x = flex as i64 * 3 + drift;
    (x + 500).div_euclid(1000).clamp(0, 3) as u8
}

/// A counter-offer: `share` thousandths of the way from the minimum up to
/// asking, rounded up to the price's step. Never below the minimum.
pub fn counter_offer(list: u32, min: u32, share: u32) -> u32 {
    let raw = min + (list - min) * share / 1000;
    ceil_to(raw, price_step(list)).min(list)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Review {
    pub stars: u8,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sale {
    pub title: String,
    pub listed: u32,
    pub sold: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub joined: u32,
    pub rating_tenths: u32,
    pub ratings: u32,
    pub sold: u32,
    pub responds: String,
    pub bio: String,
    pub reviews: Vec<Review>,
    pub sales: Vec<Sale>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Seller {
    pub name: String,
    pub colour: String,
    pub profile: Profile,
    pub reply: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Listing {
    pub item: String,
    pub title: String,
    pub list_price: u32,
    /// Hidden from the player until the round ends, and then only if skipped.
    pub min_price: u32,
    pub hours: u32,
    pub miles: u32,
    /// Hidden: the seller's flexibility, 0..=1000.
    pub flex: u32,
    pub bucket: Bucket,
    pub tone: u8,
    pub description: String,
    pub seller: Seller,
}

/// Hours since listing: equally likely to be hours, a day or two, a few days,
/// a week, two weeks or a month.
fn listing_hours(rng: &mut Rng) -> u32 {
    let spans: [(i64, i64); 6] = [(1, 5), (6, 23), (24, 71), (72, 167), (168, 335), (336, 720)];
    let (lo, hi) = *rng.pick(&spans);
    rng.between(lo, hi) as u32
}

fn noise(rng: &mut Rng) -> i64 {
    rng.between(-NOISE, NOISE) + rng.between(-NOISE, NOISE) + rng.between(-NOISE, NOISE)
}

fn lower_if(text: String, casual: bool) -> String {
    if casual {
        text.to_lowercase()
    } else {
        text
    }
}

fn neighbour(rng: &mut Rng, bucket: Bucket) -> Bucket {
    match bucket {
        Bucket::Stubborn | Bucket::Flexible => Bucket::Steady,
        Bucket::Steady => {
            if rng.chance(500) {
                Bucket::Stubborn
            } else {
                Bucket::Flexible
            }
        }
    }
}

fn profile(rng: &mut Rng, data: &Data, item: &Item, flex: u32, bucket: Bucket, casual: bool) -> Profile {
    let s = &data.sellers;
    let (lo, hi) = match bucket {
        Bucket::Stubborn => (41, 49),
        Bucket::Steady => (44, 50),
        Bucket::Flexible => (46, 50),
    };
    let rating_tenths = rng.between(lo, hi) as u32;
    let first = rng.pick(s.reviews.of(bucket)).clone();
    // One review in five comes from a buyer who saw the seller on a different
    // day: the clue points the right way most of the time, not every time.
    let second_bucket = if rng.chance(200) {
        neighbour(rng, bucket)
    } else {
        bucket
    };
    let pool: Vec<&String> = s
        .reviews
        .of(second_bucket)
        .iter()
        .filter(|r| **r != first)
        .collect();
    let mut reviews = vec![Review {
        stars: 4 + rng.below(2) as u8,
        text: first,
    }];
    if !pool.is_empty() {
        reviews.push(Review {
            stars: 4 + rng.below(2) as u8,
            text: (*rng.pick(&pool)).clone(),
        });
    }
    let others: Vec<&Item> = data.items.iter().filter(|i| i.id != item.id).collect();
    let mut sales = Vec::new();
    for _ in 0..2 {
        let other = *rng.pick(&others);
        let listed = *rng.pick(&other.prices);
        let off = (100 + 3 * flex as i64 + rng.between(-500, 500)).clamp(0, 5000) as u64;
        let raw = div_round(listed as u64 * (10_000 - off), 10_000) as u32;
        let sold = ceil_to(raw.max(1), price_step(listed)).min(listed);
        sales.push(Sale {
            title: other.short.clone(),
            listed,
            sold,
        });
    }
    let responds_bucket = if rng.chance(200) {
        neighbour(rng, bucket)
    } else {
        bucket
    };
    Profile {
        joined: rng.between(2011, 2025) as u32,
        rating_tenths,
        ratings: rng.between(3, 140) as u32,
        sold: rng.between(2, 180) as u32,
        responds: rng.pick(s.responds.of(responds_bucket)).clone(),
        bio: lower_if(rng.pick(s.bios.of(bucket)).clone(), casual),
        reviews,
        sales,
    }
}

fn reply(rng: &mut Rng, data: &Data, bucket: Bucket, list: u32, min: u32, casual: bool) -> String {
    let s = &data.sellers;
    if bucket == Bucket::Stubborn && rng.chance(500) {
        return rng.pick(&s.firm_replies).clone();
    }
    let (lo, hi) = match bucket {
        Bucket::Stubborn => (750, 1000),
        Bucket::Steady => (350, 700),
        Bucket::Flexible => (100, 450),
    };
    let counter = counter_offer(list, min, rng.between(lo, hi) as u32);
    let template: &String = rng.pick(s.replies.of(bucket));
    let line = fill(template, &[("counter", money(counter))]);
    lower_if(line, casual)
}

fn description(rng: &mut Rng, data: &Data, item: &Item, bucket: Bucket, tone: u8, casual: bool) -> String {
    let s = &data.sellers;
    let mut closer = rng.pick(s.closers.of(bucket)).clone();
    if bucket == Bucket::Stubborn && rng.chance(400) {
        closer = closer.to_uppercase();
    }
    let parts = [
        rng.pick(&item.descriptions).clone(),
        rng.pick(&s.tone_phrases[tone as usize]).clone(),
        closer,
    ];
    let text = parts
        .iter()
        .filter(|p| !p.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    lower_if(text, casual)
}

/// One listing for `item`, sold by `name`.
pub fn make_listing(rng: &mut Rng, data: &Data, item: &Item, name: &str) -> Listing {
    let flex = rng.below(1001) as u32;
    let bucket = bucket_of(flex);
    let tone = tone_of(flex, rng.between(-TONE_DRIFT, TONE_DRIFT));
    let list_price = *rng.pick(&item.prices);
    let hours = listing_hours(rng);
    let min_price = minimum_price(list_price, discount_bps(tone, flex, hours, noise(rng)));
    // A flexible seller writes in lower case half the time, everywhere they write.
    let casual = bucket == Bucket::Flexible && rng.chance(500);
    let description = description(rng, data, item, bucket, tone, casual);
    let profile = profile(rng, data, item, flex, bucket, casual);
    let reply = reply(rng, data, bucket, list_price, min_price, casual);
    Listing {
        item: item.id.clone(),
        title: item.title.clone(),
        list_price,
        min_price,
        hours,
        miles: rng.between(1, 25) as u32,
        flex,
        bucket,
        tone,
        description,
        seller: Seller {
            name: name.to_string(),
            colour: rng.pick(&data.sellers.colours).clone(),
            profile,
            reply,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buckets_split_flex_in_thirds() {
        assert_eq!(bucket_of(0), Bucket::Stubborn);
        assert_eq!(bucket_of(333), Bucket::Stubborn);
        assert_eq!(bucket_of(334), Bucket::Steady);
        assert_eq!(bucket_of(666), Bucket::Steady);
        assert_eq!(bucket_of(667), Bucket::Flexible);
        assert_eq!(bucket_of(1000), Bucket::Flexible);
    }

    #[test]
    fn round_prices_follow_the_brief() {
        for p in [5, 20, 95, 100, 125, 250, 4000] {
            assert!(price_is_round(p), "{p}");
        }
        for p in [0, 21, 99, 110, 130, 4010] {
            assert!(!price_is_round(p), "{p}");
        }
    }

    #[test]
    fn the_minimum_matches_a_hand_worked_case() {
        // $100, "offers accepted" (tone 3), flex 500, listed 4 days (96 h), no noise:
        // 200 + 3*350 + 2000*500/1000 + 400 = 2650 bps, so 26.5% off: $73.50 -> $74.
        let d = discount_bps(3, 500, 96, 0);
        assert_eq!(d, 2650);
        assert_eq!(minimum_price(100, d), 74);
        // $20 firm toaster, flex 100, listed 2 hours: 200 + 0 + 200 + 0 = 400 bps -> $19.20 -> $19.
        assert_eq!(minimum_price(20, discount_bps(0, 100, 2, 0)), 19);
    }

    #[test]
    fn the_discount_is_clamped() {
        assert_eq!(discount_bps(0, 0, 1, -5000), 0);
        assert_eq!(discount_bps(3, 1000, 700, 5000), MAX_DISCOUNT as u32);
        assert_eq!(minimum_price(1, 4500), 1);
    }

    #[test]
    fn tone_follows_flex_within_the_drift() {
        assert_eq!(tone_of(0, 0), 0);
        assert_eq!(tone_of(1000, 0), 3);
        assert_eq!(tone_of(500, 0), 2);
        // A stubborn seller can drift up to "or best offer" but never "offers accepted".
        assert!(tone_of(STEADY_FROM - 1, TONE_DRIFT) <= 2);
        // A flexible seller can drift down to plain but never "firm".
        assert!(tone_of(FLEXIBLE_FROM, -TONE_DRIFT) >= 1);
    }

    #[test]
    fn a_counter_offer_is_never_below_the_minimum_or_above_asking() {
        // $100 items count in $5 steps: 74 rounds up to 75.
        assert_eq!(counter_offer(100, 74, 0), 75);
        assert_eq!(counter_offer(100, 74, 1000), 100);
        // 74 + 26*500/1000 = 87, up to 90.
        assert_eq!(counter_offer(100, 74, 500), 90);
        // $40 items count in dollars.
        assert_eq!(counter_offer(40, 31, 0), 31);
        // $4,000 car, minimum $3,111: rounds up to $50 steps.
        assert_eq!(counter_offer(4000, 3111, 0), 3150);
    }

    #[test]
    fn older_listings_never_get_a_smaller_bonus() {
        let mut last = 0;
        for h in 1..=720 {
            let b = age_bonus(h);
            assert!(b >= last);
            last = b;
        }
    }
}
