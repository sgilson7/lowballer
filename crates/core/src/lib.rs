//! lowball_core: every rule of Lowballer.
//!
//! Builder requirement: this crate performs no input or output. It takes
//! values and returns values, so `cargo test` reaches every rule without a
//! browser. `tests/boundary.rs` enforces that, along with the dependency
//! allowlist and the reference ban on floats, hash maps and clocks.
//!
//! - `data`: the content files (items, seller voices, copy) and their checks.
//! - `rng`: the seeded generator every random choice comes from.
//! - `market`: how a listing, its seller and the seller's hidden minimum are made.
//! - `game`: the state, the limits on each screen, offers and scoring.
//! - `view`: everything the page draws, already worded and formatted.
//! - `api`: the one door the shim calls through.

pub mod api;
pub mod data;
pub mod game;
pub mod market;
pub mod rng;
pub mod text;
pub mod view;
