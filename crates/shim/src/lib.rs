//! lowball_shim: forwards strings between the page and `lowball_core::api`.
//!
//! Builder requirement: the shim decides nothing. Each function is one call
//! into the core. `crates/core/tests/boundary.rs` fails if a branch, a loop or
//! the `?` operator appears here; if a feature seems to need one, the rule
//! belongs in the core.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn start(content: &str, seed: &str) -> String {
    lowball_core::api::start(content, seed)
}

#[wasm_bindgen]
pub fn act(content: &str, state: &str, action: &str) -> String {
    lowball_core::api::act(content, state, action)
}

#[wasm_bindgen]
pub fn hint(content: &str, state: &str, listing: &str, amount: &str) -> String {
    lowball_core::api::hint(content, state, listing, amount)
}
