//! The one door into the core. Every function takes strings and returns a JSON
//! string: `{"ok": ...}` on success or `{"error": "..."}` with a message the
//! page can show as it is. The shim forwards these calls and nothing else.
//!
//! The page keeps the game state between calls as the JSON this module
//! returned, and passes it back. `content` is the three data files combined:
//! `{"items": [...], "sellers": {...}, "copy": {...}}`.

use serde::Serialize;

use crate::data::{self, Data};
use crate::game::{self, Action, Game};
use crate::view;

fn ok<T: Serialize>(value: &T) -> String {
    serde_json::json!({ "ok": value }).to_string()
}

fn error(message: String) -> String {
    serde_json::json!({ "error": message }).to_string()
}

#[derive(Serialize)]
struct Step<'a> {
    state: &'a Game,
    view: view::View,
}

fn step(data: &Data, game: &Game) -> String {
    ok(&Step {
        state: game,
        view: view::view(data, game),
    })
}

/// A seed as the page sends it: decimal digits. Anything else seeds with 0.
pub fn parse_seed(text: &str) -> u64 {
    text.trim().parse().unwrap_or(0)
}

/// A new game, on its title screen.
pub fn start(content: &str, seed: &str) -> String {
    match data::parse(content) {
        Ok(d) => step(&d, &game::new_game(&d, parse_seed(seed))),
        Err(e) => error(e),
    }
}

fn load(content: &str, state: &str) -> Result<(Data, Game), String> {
    let d = data::parse(content)?;
    let g: Game = serde_json::from_str(state).map_err(|_| d.copy.error("bad-state"))?;
    Ok((d, g))
}

/// The game after one action: `{"kind": "start"}`, `{"kind": "peek", "listing": "2"}`,
/// `{"kind": "message", "listing": "2"}`, `{"kind": "offer", "listing": "2", "amount": "80"}`
/// or `{"kind": "next"}`.
pub fn act(content: &str, state: &str, action: &str) -> String {
    let (d, g) = match load(content, state) {
        Ok(x) => x,
        Err(e) => return error(e),
    };
    let a: Action = match serde_json::from_str(action) {
        Ok(a) => a,
        Err(_) => return error(d.copy.error("bad-action")),
    };
    match game::apply(&g, &a) {
        Ok(next) => step(&d, &next),
        Err(code) => error(d.copy.error(code)),
    }
}

/// The line under the offer box while the player types `amount` on `listing`.
pub fn hint(content: &str, state: &str, listing: &str, amount: &str) -> String {
    let (d, g) = match load(content, state) {
        Ok(x) => x,
        Err(e) => return error(e),
    };
    match game::listing_index(listing) {
        Ok(i) => ok(&view::hint(&d, &g, i, amount)),
        Err(code) => error(d.copy.error(code)),
    }
}
