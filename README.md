# Lowballer

A 2D browser game about lowballing sellers on a neighbourhood marketplace. Each of three screens shows six listings. Make an offer on five and skip one. Every seller has a secret lowest price: offer at or above it and you save the difference; offer below it and you save nothing. Read the listing, inspect two profiles and message two sellers per screen (never both on one listing), and see how far under asking you can go.

Play it at <https://sgilson7.github.io/lowballer/>.

## How the sellers work

Each seller has one hidden flexibility. It sets the tone phrase in the description ("Price is firm" to "Offers accepted"), the voice they write in, their profile (bio, reviews, response time, what their recent sales went for), how they answer a message, and, with the listing's age and some noise, the lowest price they'll take. The numbers are in `crates/core/src/market.rs`.

## Build, test and serve

Built on [Builder](../Builder/builder-setting): the rules live in a Rust core with no I/O, a WebAssembly shim only forwards strings, and a static page draws what the core returns.

```bash
./scripts/setup.sh          # once: Python venv with Playwright and three browsers
cargo test --workspace      # core and boundary tests, in seconds
python3 scripts/break_tests.py   # break each rule once; its test must fail
python3 tools/art.py        # redraw the TikZ art into web/art/ (needs pdflatex, pdftocairo)
./scripts/build.sh          # release WebAssembly build into dist/
python3 scripts/serve.py    # http://127.0.0.1:8000/
./scripts/check.sh          # everything, before a deploy
```

Needs Rust with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.127, and Python 3.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core` | every rule: listings, sellers, minimums, limits, scoring, the view the page draws |
| `crates/shim` | three `wasm-bindgen` functions, each one call into the core |
| `web/` | the static page and the art |
| `data/` | items, seller voices and every word on screen |
| `tools/art.py` | the TikZ art generator; sources land in `tools/art/` |
| `tests/` | the art check and the three-engine browser check |
