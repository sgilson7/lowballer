# Measurements

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.95.0; wasm-bindgen 0.2.127 | 2026-10-10 |
| Core tests | `cargo test --workspace` | 41 tests pass (13 unit, 11 boundary, 17 game), under a second | 2026-10-10 |
| Break tests | `python3 scripts/break_tests.py` | 14 of 14 breaks caught | 2026-10-10 |
| Full check | `./scripts/check.sh` | BUILDER CHECK PASSED in chromium, firefox, webkit | 2026-10-10 |
| Art | `python3 tools/art.py` | 34 pictures, about 15 s | 2026-10-10 |
