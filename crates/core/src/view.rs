//! Everything the page draws, worded and formatted here so the page only
//! places it. Hidden values (a seller's flex and minimum) never enter a view,
//! except the minimum of a skipped listing once its screen is over.

use serde::Serialize;

use crate::data::{Copy, Data};
use crate::game::{
    self, message_blocker, offer_blocker, outcomes, peek_blocker, Game, Outcome, Phase, LISTINGS, MESSAGES,
    OFFERS, PEEKS, ROUNDS,
};
use crate::market::{price_step, Listing};
use crate::text::{ceil_to, div_round, fill, money, tenths};

#[derive(Debug, Serialize)]
pub struct View {
    pub phase: Phase,
    pub top: Top,
    pub title: Option<TitleScreen>,
    pub browse: Option<Browse>,
    pub summary: Option<Summary>,
    #[serde(rename = "final")]
    pub final_screen: Option<FinalScreen>,
}

#[derive(Debug, Serialize)]
pub struct Top {
    pub brand: String,
    pub brand_tag: String,
    pub screen: String,
    pub saved_label: String,
    pub saved: String,
}

#[derive(Debug, Serialize)]
pub struct TitleScreen {
    pub name: String,
    pub tagline: String,
    pub how: Vec<String>,
    pub button: String,
    pub art_alt: String,
}

#[derive(Debug, Serialize)]
pub struct Meter {
    pub label: String,
    pub count: String,
    pub pips: Vec<bool>,
}

#[derive(Debug, Serialize)]
pub struct Badge {
    pub text: String,
    pub kind: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Button {
    pub label: String,
    pub enabled: bool,
    /// Why it is disabled, when the reason is not already on screen.
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct ReviewView {
    pub stars: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct SaleView {
    pub title: String,
    pub line: String,
}

#[derive(Debug, Serialize)]
pub struct ProfileView {
    pub facts: Vec<String>,
    pub responds: String,
    pub bio: String,
    pub reviews_head: String,
    pub reviews: Vec<ReviewView>,
    pub sales_head: String,
    pub sales: Vec<SaleView>,
}

#[derive(Debug, Serialize)]
pub struct Chat {
    pub you: String,
    pub question: String,
    pub reply: String,
}

#[derive(Debug, Serialize)]
pub struct Offer {
    pub open: bool,
    pub currency: String,
    pub sent: String,
    pub label: String,
    pub button: String,
    pub min: String,
    pub max: String,
    pub step: String,
    pub start: String,
    pub hint: String,
}

#[derive(Debug, Serialize)]
pub struct Card {
    pub key: String,
    pub art: String,
    pub alt: String,
    pub price: String,
    pub title: String,
    pub meta: String,
    pub done: bool,
    pub badges: Vec<Badge>,
    pub description_head: String,
    pub description: String,
    pub seller_head: String,
    pub seller: String,
    pub initial: String,
    pub colour: String,
    pub peek: Button,
    pub message: Button,
    pub profile: Option<ProfileView>,
    pub chat: Option<Chat>,
    pub offer: Offer,
    pub close: String,
}

#[derive(Debug, Serialize)]
pub struct Browse {
    pub heading: String,
    pub how: String,
    pub meters: Vec<Meter>,
    pub cards: Vec<Card>,
}

#[derive(Debug, Serialize)]
pub struct Row {
    pub art: String,
    pub alt: String,
    pub title: String,
    pub asking: String,
    pub status: &'static str,
    pub label: String,
    pub line: String,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub title: String,
    pub rows: Vec<Row>,
    pub total_label: String,
    pub total: String,
    pub button: String,
}

#[derive(Debug, Serialize)]
pub struct Stat {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct FinalScreen {
    pub eyebrow: String,
    pub pct: String,
    pub line: String,
    pub rank: String,
    pub rank_line: String,
    pub stats: Vec<Stat>,
    pub rounds: Vec<Stat>,
    pub button: String,
    pub art_alt: String,
}

fn n(v: usize) -> String {
    v.to_string()
}

fn art(l: &Listing) -> String {
    format!("art/items/{}.png", l.item)
}

/// "3 hours ago", "1 day ago", "2 weeks ago".
pub fn age(copy: &Copy, hours: u32) -> String {
    let (count, one, many) = if hours < 24 {
        (hours, &copy.age_hour, &copy.age_hours)
    } else if hours < 24 * 14 {
        (hours / 24, &copy.age_day, &copy.age_days)
    } else {
        (hours / (24 * 7), &copy.age_week, &copy.age_weeks)
    };
    fill(if count == 1 { one } else { many }, &[("n", count.to_string())])
}

fn meta(copy: &Copy, l: &Listing) -> String {
    fill(
        &copy.meta,
        &[
            ("listed", fill(&copy.listed, &[("age", age(copy, l.hours))])),
            ("miles", fill(&copy.miles, &[("n", l.miles.to_string())])),
        ],
    )
}

fn meter(copy: &Copy, label: &str, used: usize, total: usize) -> Meter {
    Meter {
        label: label.to_string(),
        count: fill(&copy.meter_count, &[("used", n(used)), ("total", n(total))]),
        pips: (0..total).map(|k| k < used).collect(),
    }
}

/// Reasons that are already visible (the profile or chat on screen, the offer
/// note) are not repeated under a disabled button.
fn note(copy: &Copy, code: Option<&'static str>) -> String {
    match code {
        None | Some("already-peeked") | Some("already-messaged") | Some("already-offered") => String::new(),
        Some(c) => copy.error(c),
    }
}

fn stars(k: u8) -> String {
    "★".repeat(k as usize) + &"☆".repeat(5 - k as usize)
}

/// The share of asking an offer of `amount` takes off, in whole percent, rounded half up.
pub fn percent_below(list: u32, amount: u32) -> u32 {
    div_round((list - amount.min(list)) as u64 * 100, list as u64) as u32
}

/// The line under the offer box as the player types.
pub fn hint(data: &Data, game: &Game, i: usize, amount: &str) -> String {
    let copy = &data.copy;
    let list = game.current().listings[i].list_price;
    match game::parse_amount(amount) {
        None => copy.error("bad-amount"),
        Some(a) if a > list => copy.error("over-asking"),
        Some(a) if a == list => copy.offer_full.clone(),
        Some(a) => fill(&copy.offer_hint, &[("pct", percent_below(list, a).to_string())]),
    }
}

/// Where the offer slider starts: 85% of asking, rounded up to the price's step.
pub fn offer_start(list: u32) -> u32 {
    ceil_to((list * 85).div_ceil(100), price_step(list)).min(list)
}

/// The slider's lower end: 30% of asking, at least $1.
pub fn offer_floor(list: u32) -> u32 {
    (list * 3 / 10).max(1)
}

fn profile_view(copy: &Copy, l: &Listing) -> ProfileView {
    let p = &l.seller.profile;
    ProfileView {
        facts: vec![
            fill(&copy.joined, &[("year", p.joined.to_string())]),
            fill(
                &copy.rating,
                &[
                    ("rating", tenths(p.rating_tenths)),
                    ("count", p.ratings.to_string()),
                ],
            ),
            fill(&copy.sold_count, &[("n", p.sold.to_string())]),
        ],
        responds: p.responds.clone(),
        bio: p.bio.clone(),
        reviews_head: copy.reviews_head.clone(),
        reviews: p
            .reviews
            .iter()
            .map(|r| ReviewView {
                stars: stars(r.stars),
                text: r.text.clone(),
            })
            .collect(),
        sales_head: copy.sales_head.clone(),
        sales: p
            .sales
            .iter()
            .map(|s| SaleView {
                title: s.title.clone(),
                line: fill(
                    &copy.sale_line,
                    &[("listed", money(s.listed)), ("sold", money(s.sold))],
                ),
            })
            .collect(),
    }
}

fn card(data: &Data, game: &Game, i: usize) -> Card {
    let copy = &data.copy;
    let r = game.current();
    let l = &r.listings[i];
    let offered = r.offers[i];
    let peeked = r.peeked.contains(&i);
    let messaged = r.messaged.contains(&i);
    let mut badges = Vec::new();
    if let Some(o) = offered {
        badges.push(Badge {
            text: fill(&copy.badge_offer, &[("amount", money(o))]),
            kind: "offer",
        });
    }
    if peeked {
        badges.push(Badge {
            text: copy.badge_peeked.clone(),
            kind: "intel",
        });
    }
    if messaged {
        badges.push(Badge {
            text: copy.badge_messaged.clone(),
            kind: "intel",
        });
    }
    let pb = peek_blocker(game, i);
    let mb = message_blocker(game, i);
    let ob = offer_blocker(game, i);
    let list = l.list_price;
    Card {
        key: n(i),
        art: art(l),
        alt: fill(&copy.photo_alt, &[("title", l.title.clone())]),
        price: money(list),
        title: l.title.clone(),
        meta: meta(copy, l),
        done: offered.is_some(),
        badges,
        description_head: copy.description_head.clone(),
        description: l.description.clone(),
        seller_head: copy.seller_head.clone(),
        seller: l.seller.name.clone(),
        initial: l
            .seller
            .name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default(),
        colour: l.seller.colour.clone(),
        peek: Button {
            label: copy.inspect_profile.clone(),
            enabled: pb.is_none(),
            note: note(copy, pb),
        },
        message: Button {
            label: copy.message_seller.clone(),
            enabled: mb.is_none(),
            note: note(copy, mb),
        },
        profile: peeked.then(|| profile_view(copy, l)),
        chat: messaged.then(|| Chat {
            you: copy.chat_you.clone(),
            question: copy.question.clone(),
            reply: l.seller.reply.clone(),
        }),
        offer: Offer {
            open: ob.is_none(),
            currency: copy.currency.clone(),
            sent: offered
                .map(|o| fill(&copy.offer_sent, &[("amount", money(o))]))
                .unwrap_or_default(),
            label: copy.offer_label.clone(),
            button: copy.offer_button.clone(),
            min: offer_floor(list).to_string(),
            max: list.to_string(),
            step: price_step(list).to_string(),
            start: offer_start(list).to_string(),
            hint: fill(
                &copy.offer_hint,
                &[("pct", percent_below(list, offer_start(list)).to_string())],
            ),
        },
        close: copy.close.clone(),
    }
}

fn summary(data: &Data, game: &Game) -> Summary {
    let copy = &data.copy;
    let r = game.current();
    let rows = r
        .listings
        .iter()
        .zip(outcomes(game, game.round))
        .map(|(l, o)| {
            let (status, label, line) = match o {
                Outcome::Accepted { offer, saved } => (
                    "accepted",
                    &copy.status_accepted,
                    fill(
                        &copy.line_accepted,
                        &[
                            ("offer", money(offer)),
                            ("list", money(l.list_price)),
                            ("saved", money(saved)),
                        ],
                    ),
                ),
                Outcome::Declined { offer } => (
                    "declined",
                    &copy.status_declined,
                    fill(&copy.line_declined, &[("offer", money(offer))]),
                ),
                // A summary only shows a finished screen, so nothing is pending.
                Outcome::Skipped { min } => (
                    "skipped",
                    &copy.status_skipped,
                    fill(&copy.line_skipped, &[("min", money(min))]),
                ),
                Outcome::Pending => ("skipped", &copy.status_skipped, String::new()),
            };
            Row {
                art: art(l),
                alt: fill(&copy.photo_alt, &[("title", l.title.clone())]),
                title: l.title.clone(),
                asking: money(l.list_price),
                status,
                label: label.clone(),
                line,
            }
        })
        .collect();
    let (saved, _) = game::round_totals(game, game.round);
    Summary {
        title: fill(&copy.summary_title, &[("n", n(game.round + 1))]),
        rows,
        total_label: copy.summary_total.clone(),
        total: money(saved),
        button: if game.round + 1 < ROUNDS {
            copy.next_button.clone()
        } else {
            copy.final_button.clone()
        },
    }
}

fn final_screen(data: &Data, game: &Game) -> FinalScreen {
    let copy = &data.copy;
    let t = game::tally(game);
    let pct = tenths(t.pct_tenths);
    let rank = &copy.ranks[t.rank];
    FinalScreen {
        eyebrow: copy.final_eyebrow.clone(),
        pct: format!("{pct}%"),
        line: fill(&copy.final_line, &[("pct", pct)]),
        rank: rank.name.clone(),
        rank_line: rank.line.clone(),
        stats: vec![
            Stat {
                label: copy.stat_saved.clone(),
                value: money(t.saved),
            },
            Stat {
                label: copy.stat_asking.clone(),
                value: money(t.asking),
            },
            Stat {
                label: copy.stat_accepted.clone(),
                value: fill(
                    &copy.accepted_of,
                    &[("n", t.accepted.to_string()), ("total", t.offered.to_string())],
                ),
            },
        ],
        rounds: (0..ROUNDS)
            .map(|r| {
                let (s, a) = game::round_totals(game, r);
                Stat {
                    label: fill(&copy.round_line, &[("n", n(r + 1))]),
                    value: fill(&copy.round_value, &[("saved", money(s)), ("asking", money(a))]),
                }
            })
            .collect(),
        button: copy.again_button.clone(),
        art_alt: copy.final_art_alt.clone(),
    }
}

pub fn view(data: &Data, game: &Game) -> View {
    let copy = &data.copy;
    let r = game.current();
    View {
        phase: game.phase,
        top: Top {
            brand: copy.brand.clone(),
            brand_tag: copy.brand_tag.clone(),
            screen: fill(&copy.screen_of, &[("n", n(game.round + 1)), ("total", n(ROUNDS))]),
            saved_label: copy.saved_label.clone(),
            saved: money(game::saved_so_far(game)),
        },
        title: (game.phase == Phase::Title).then(|| TitleScreen {
            name: copy.game_name.clone(),
            tagline: copy.tagline.clone(),
            how: copy.how_to_play.clone(),
            button: copy.start_button.clone(),
            art_alt: copy.title_art_alt.clone(),
        }),
        browse: (game.phase == Phase::Browse).then(|| Browse {
            heading: copy.browse_heading.clone(),
            how: copy.browse_how.clone(),
            meters: vec![
                meter(copy, &copy.meter_offers, r.offers_made(), OFFERS),
                meter(copy, &copy.meter_peeks, r.peeked.len(), PEEKS),
                meter(copy, &copy.meter_messages, r.messaged.len(), MESSAGES),
            ],
            cards: (0..LISTINGS).map(|i| card(data, game, i)).collect(),
        }),
        summary: (game.phase == Phase::Summary).then(|| summary(data, game)),
        final_screen: (game.phase == Phase::Final).then(|| final_screen(data, game)),
    }
}
