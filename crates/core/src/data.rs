//! The content files: `data/items.json`, `data/sellers.json` and
//! `data/copy.json`, passed in together by the page as one JSON object. Every
//! word a player reads comes from these files; this module checks that they
//! hold everything the rules will reach for, so a gap fails at load and not in
//! the middle of a game.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::game::{ERROR_CODES, LISTINGS, RANK_TENTHS, ROUNDS};
use crate::market::{price_is_round, Bucket, TONES};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    /// Also the picture's file name: `art/items/<id>.png`.
    pub id: String,
    pub title: String,
    /// The name used in another seller's "recently sold" list.
    pub short: String,
    /// Asking prices a seller may choose from.
    pub prices: Vec<u32>,
    pub descriptions: Vec<String>,
}

/// One pool of lines per seller temperament.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ByBucket {
    pub stubborn: Vec<String>,
    pub steady: Vec<String>,
    pub flexible: Vec<String>,
}

impl ByBucket {
    pub fn of(&self, bucket: Bucket) -> &[String] {
        match bucket {
            Bucket::Stubborn => &self.stubborn,
            Bucket::Steady => &self.steady,
            Bucket::Flexible => &self.flexible,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sellers {
    pub names: Vec<String>,
    /// Avatar colour names; the page has a class for each.
    pub colours: Vec<String>,
    /// Four tiers, firm to open: the phrase a description carries about price.
    pub tone_phrases: Vec<Vec<String>>,
    pub closers: ByBucket,
    pub bios: ByBucket,
    pub reviews: ByBucket,
    pub responds: ByBucket,
    /// Replies that name a price as `{counter}`.
    pub replies: ByBucket,
    /// Replies a stubborn seller gives without naming a price.
    pub firm_replies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rank {
    pub name: String,
    pub line: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Copy {
    pub game_name: String,
    pub tagline: String,
    pub how_to_play: Vec<String>,
    pub start_button: String,
    pub title_art_alt: String,
    pub brand: String,
    pub brand_tag: String,
    pub screen_of: String,
    pub saved_label: String,
    pub browse_heading: String,
    pub browse_how: String,
    pub meter_offers: String,
    pub meter_peeks: String,
    pub meter_messages: String,
    pub meter_count: String,
    pub age_hour: String,
    pub age_hours: String,
    pub age_day: String,
    pub age_days: String,
    pub age_week: String,
    pub age_weeks: String,
    pub listed: String,
    pub miles: String,
    pub meta: String,
    pub badge_offer: String,
    pub badge_peeked: String,
    pub badge_messaged: String,
    pub photo_alt: String,
    pub inspect_profile: String,
    pub message_seller: String,
    pub close: String,
    pub description_head: String,
    pub seller_head: String,
    pub joined: String,
    pub rating: String,
    pub sold_count: String,
    pub reviews_head: String,
    pub sales_head: String,
    pub sale_line: String,
    pub question: String,
    pub chat_you: String,
    pub offer_label: String,
    pub currency: String,
    pub offer_button: String,
    pub offer_hint: String,
    pub offer_full: String,
    pub offer_sent: String,
    pub summary_title: String,
    pub status_accepted: String,
    pub status_declined: String,
    pub status_skipped: String,
    pub line_accepted: String,
    pub line_declined: String,
    pub line_skipped: String,
    pub summary_total: String,
    pub next_button: String,
    pub final_button: String,
    pub final_eyebrow: String,
    pub final_line: String,
    pub final_art_alt: String,
    pub stat_saved: String,
    pub stat_asking: String,
    pub stat_accepted: String,
    pub accepted_of: String,
    pub round_line: String,
    pub round_value: String,
    pub again_button: String,
    pub errors: BTreeMap<String, String>,
    pub ranks: Vec<Rank>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub items: Vec<Item>,
    pub sellers: Sellers,
    pub copy: Copy,
}

impl Copy {
    pub fn error(&self, code: &str) -> String {
        self.errors.get(code).cloned().unwrap_or_else(|| code.to_string())
    }
}

fn nonempty(pool: &[String], what: &str) -> Result<(), String> {
    if pool.is_empty() || pool.iter().all(|s| s.trim().is_empty()) {
        return Err(format!("{what} has no lines."));
    }
    Ok(())
}

fn each_bucket(pools: &ByBucket, what: &str) -> Result<(), String> {
    nonempty(&pools.stubborn, &format!("{what}.stubborn"))?;
    nonempty(&pools.steady, &format!("{what}.steady"))?;
    nonempty(&pools.flexible, &format!("{what}.flexible"))
}

/// Parses the combined content and checks everything the rules rely on.
pub fn parse(json: &str) -> Result<Data, String> {
    let data: Data = serde_json::from_str(json).map_err(|e| format!("The content could not be read: {e}"))?;
    let needed = ROUNDS * LISTINGS;
    if data.items.len() < needed {
        return Err(format!(
            "items.json has {} items; a game needs {needed}.",
            data.items.len()
        ));
    }
    let mut ids: Vec<&str> = data.items.iter().map(|i| i.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    if ids.len() != data.items.len() {
        return Err("items.json repeats an id.".into());
    }
    for item in &data.items {
        if item.prices.is_empty() || item.prices.iter().any(|p| !price_is_round(*p)) {
            return Err(format!(
                "{}: every asking price must be round (see market::price_is_round).",
                item.id
            ));
        }
        nonempty(&item.descriptions, &item.id)?;
    }
    let s = &data.sellers;
    if s.names.len() < needed {
        return Err(format!(
            "sellers.json has {} names; a game needs {needed}.",
            s.names.len()
        ));
    }
    nonempty(&s.colours, "colours")?;
    if s.tone_phrases.len() != TONES {
        return Err(format!("tone_phrases needs {TONES} tiers, firm to open."));
    }
    for (tier, pool) in s.tone_phrases.iter().enumerate() {
        if pool.is_empty() {
            return Err(format!("tone_phrases tier {tier} is empty."));
        }
    }
    each_bucket(&s.closers, "closers")?;
    each_bucket(&s.bios, "bios")?;
    each_bucket(&s.reviews, "reviews")?;
    each_bucket(&s.responds, "responds")?;
    each_bucket(&s.replies, "replies")?;
    nonempty(&s.firm_replies, "firm_replies")?;
    for b in [Bucket::Stubborn, Bucket::Steady, Bucket::Flexible] {
        if s.replies.of(b).iter().any(|r| !r.contains("{counter}")) {
            return Err("Every line in replies must name the price as {counter}.".into());
        }
    }
    for code in ERROR_CODES {
        if !data.copy.errors.contains_key(*code) {
            return Err(format!("copy.json has no message for the error \"{code}\"."));
        }
    }
    if data.copy.ranks.len() != RANK_TENTHS.len() + 1 {
        return Err(format!("copy.json needs {} ranks.", RANK_TENTHS.len() + 1));
    }
    Ok(data)
}
