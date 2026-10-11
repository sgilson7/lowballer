//! The game: three screens of six listings. On each screen the player makes
//! five offers and skips one, may inspect two profiles and message two sellers,
//! and never does both on one listing. An accepted offer saves asking minus the
//! offer; a declined one saves nothing and still counts.

use serde::{Deserialize, Serialize};

use crate::data::Data;
use crate::market::{make_listing, Listing};
use crate::rng::Rng;
use crate::text::div_round;

pub const ROUNDS: usize = 3;
pub const LISTINGS: usize = 6;
pub const OFFERS: usize = 5;
pub const PEEKS: usize = 2;
pub const MESSAGES: usize = 2;

/// Rank boundaries for the final percentage, in tenths of a percent.
pub const RANK_TENTHS: [u32; 4] = [50, 120, 200, 300];

/// Every error an action can return. `data/copy.json` must word each one.
pub const ERROR_CODES: &[&str] = &[
    "bad-action",
    "bad-state",
    "not-browsing",
    "no-such-listing",
    "already-peeked",
    "already-messaged",
    "used-message-here",
    "used-peek-here",
    "already-offered",
    "no-peeks-left",
    "no-messages-left",
    "no-offers-left",
    "bad-amount",
    "over-asking",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Title,
    Browse,
    Summary,
    Final,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub listings: Vec<Listing>,
    pub peeked: Vec<usize>,
    pub messaged: Vec<usize>,
    pub offers: Vec<Option<u32>>,
}

impl Round {
    pub fn offers_made(&self) -> usize {
        self.offers.iter().filter(|o| o.is_some()).count()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub phase: Phase,
    pub round: usize,
    pub rounds: Vec<Round>,
}

impl Game {
    pub fn current(&self) -> &Round {
        &self.rounds[self.round]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Action {
    Start,
    Peek { listing: String },
    Message { listing: String },
    Offer { listing: String, amount: String },
    Next,
}

/// A new game from `seed`: eighteen different items, eighteen different sellers.
pub fn new_game(data: &Data, seed: u64) -> Game {
    let mut rng = Rng::new(seed);
    let mut items: Vec<usize> = (0..data.items.len()).collect();
    rng.shuffle(&mut items);
    let mut names = data.sellers.names.clone();
    rng.shuffle(&mut names);
    let mut rounds = Vec::new();
    for r in 0..ROUNDS {
        let mut listings = Vec::new();
        for k in 0..LISTINGS {
            let n = r * LISTINGS + k;
            listings.push(make_listing(&mut rng, data, &data.items[items[n]], &names[n]));
        }
        rounds.push(Round {
            listings,
            peeked: Vec::new(),
            messaged: Vec::new(),
            offers: vec![None; LISTINGS],
        });
    }
    Game {
        phase: Phase::Title,
        round: 0,
        rounds,
    }
}

/// A listing number as the page sends it.
pub fn listing_index(text: &str) -> Result<usize, &'static str> {
    match text.trim().parse::<usize>() {
        Ok(i) if i < LISTINGS => Ok(i),
        _ => Err("no-such-listing"),
    }
}

/// Why the profile on listing `i` cannot be inspected now, if it cannot.
pub fn peek_blocker(game: &Game, i: usize) -> Option<&'static str> {
    if game.phase != Phase::Browse {
        return Some("not-browsing");
    }
    let r = game.current();
    if r.peeked.contains(&i) {
        Some("already-peeked")
    } else if r.messaged.contains(&i) {
        Some("used-message-here")
    } else if r.offers[i].is_some() {
        Some("already-offered")
    } else if r.peeked.len() >= PEEKS {
        Some("no-peeks-left")
    } else {
        None
    }
}

/// Why the seller of listing `i` cannot be messaged now, if they cannot.
pub fn message_blocker(game: &Game, i: usize) -> Option<&'static str> {
    if game.phase != Phase::Browse {
        return Some("not-browsing");
    }
    let r = game.current();
    if r.messaged.contains(&i) {
        Some("already-messaged")
    } else if r.peeked.contains(&i) {
        Some("used-peek-here")
    } else if r.offers[i].is_some() {
        Some("already-offered")
    } else if r.messaged.len() >= MESSAGES {
        Some("no-messages-left")
    } else {
        None
    }
}

/// Why no offer can be made on listing `i` now, if none can.
pub fn offer_blocker(game: &Game, i: usize) -> Option<&'static str> {
    if game.phase != Phase::Browse {
        return Some("not-browsing");
    }
    let r = game.current();
    if r.offers[i].is_some() {
        Some("already-offered")
    } else if r.offers_made() >= OFFERS {
        Some("no-offers-left")
    } else {
        None
    }
}

/// Whole dollars as a person types them: "80", "$80", "1,250". Nothing else.
pub fn parse_amount(text: &str) -> Option<u32> {
    let t = text.trim();
    let t = t.strip_prefix('$').unwrap_or(t).trim();
    let digits: String = t.chars().filter(|c| *c != ',').collect();
    if digits.is_empty() || digits.len() > 7 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    if n == 0 {
        None
    } else {
        Some(n)
    }
}

/// An offer of `amount` on listing `i`, checked as if it were being sent.
pub fn check_offer(game: &Game, i: usize, amount: &str) -> Result<u32, &'static str> {
    if let Some(code) = offer_blocker(game, i) {
        return Err(code);
    }
    let n = parse_amount(amount).ok_or("bad-amount")?;
    if n > game.current().listings[i].list_price {
        return Err("over-asking");
    }
    Ok(n)
}

/// The game after `action`, or the code of the reason it was refused.
pub fn apply(game: &Game, action: &Action) -> Result<Game, &'static str> {
    let mut g = game.clone();
    match action {
        Action::Start => {
            if g.phase != Phase::Title {
                return Err("bad-action");
            }
            g.phase = Phase::Browse;
        }
        Action::Peek { listing } => {
            let i = listing_index(listing)?;
            if let Some(code) = peek_blocker(&g, i) {
                return Err(code);
            }
            g.rounds[g.round].peeked.push(i);
        }
        Action::Message { listing } => {
            let i = listing_index(listing)?;
            if let Some(code) = message_blocker(&g, i) {
                return Err(code);
            }
            g.rounds[g.round].messaged.push(i);
        }
        Action::Offer { listing, amount } => {
            let i = listing_index(listing)?;
            let n = check_offer(&g, i, amount)?;
            let round = &mut g.rounds[g.round];
            round.offers[i] = Some(n);
            if round.offers_made() == OFFERS {
                g.phase = Phase::Summary;
            }
        }
        Action::Next => {
            if g.phase != Phase::Summary {
                return Err("bad-action");
            }
            if g.round + 1 < ROUNDS {
                g.round += 1;
                g.phase = Phase::Browse;
            } else {
                g.phase = Phase::Final;
            }
        }
    }
    Ok(g)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Accepted {
        offer: u32,
        saved: u32,
    },
    Declined {
        offer: u32,
    },
    Skipped {
        min: u32,
    },
    /// The screen is still being played and this listing has no offer yet.
    Pending,
}

pub fn outcome(listing: &Listing, offer: Option<u32>, finished: bool) -> Outcome {
    match offer {
        Some(o) if o >= listing.min_price => Outcome::Accepted {
            offer: o,
            saved: listing.list_price - o,
        },
        Some(o) => Outcome::Declined { offer: o },
        None if finished => Outcome::Skipped {
            min: listing.min_price,
        },
        None => Outcome::Pending,
    }
}

/// Whether round `r` has been played to the end.
pub fn finished(game: &Game, r: usize) -> bool {
    r < game.round || (r == game.round && matches!(game.phase, Phase::Summary | Phase::Final))
}

pub fn outcomes(game: &Game, r: usize) -> Vec<Outcome> {
    let round = &game.rounds[r];
    let done = finished(game, r);
    round
        .listings
        .iter()
        .zip(&round.offers)
        .map(|(l, o)| outcome(l, *o, done))
        .collect()
}

/// Dollars saved on round `r`, and the asking prices of the items bid on.
pub fn round_totals(game: &Game, r: usize) -> (u32, u32) {
    let round = &game.rounds[r];
    let saved = outcomes(game, r)
        .iter()
        .map(|o| match o {
            Outcome::Accepted { saved, .. } => *saved,
            _ => 0,
        })
        .sum();
    let asking = round
        .listings
        .iter()
        .zip(&round.offers)
        .filter(|(_, o)| o.is_some())
        .map(|(l, _)| l.list_price)
        .sum();
    (saved, asking)
}

/// Saved across the screens already finished.
pub fn saved_so_far(game: &Game) -> u32 {
    (0..ROUNDS)
        .filter(|r| finished(game, *r))
        .map(|r| round_totals(game, r).0)
        .sum()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tally {
    pub saved: u32,
    pub asking: u32,
    pub accepted: u32,
    pub offered: u32,
    /// Saved as a share of asking, in tenths of a percent, rounded half up.
    pub pct_tenths: u32,
    pub rank: usize,
}

pub fn tally(game: &Game) -> Tally {
    let mut t = Tally {
        saved: 0,
        asking: 0,
        accepted: 0,
        offered: 0,
        pct_tenths: 0,
        rank: 0,
    };
    for r in (0..ROUNDS).filter(|r| finished(game, *r)) {
        let (s, a) = round_totals(game, r);
        t.saved += s;
        t.asking += a;
        for o in outcomes(game, r) {
            match o {
                Outcome::Accepted { .. } => {
                    t.accepted += 1;
                    t.offered += 1;
                }
                Outcome::Declined { .. } => t.offered += 1,
                _ => {}
            }
        }
    }
    if t.asking > 0 {
        t.pct_tenths = div_round(t.saved as u64 * 1000, t.asking as u64) as u32;
    }
    t.rank = RANK_TENTHS.iter().filter(|b| t.pct_tenths >= **b).count();
    t
}
