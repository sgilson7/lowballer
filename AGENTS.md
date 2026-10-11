# Instructions for a coding agent in a Builder project

`CLAUDE.md` holds the same rules for agents that read that file instead.

1. Read `BRIEF.md`, `PLAN.md` and `BUILDER-SPEC.md` before changing code.
2. The core (`crates/core`) owns every application decision and performs no input or output.
3. The shim (`crates/shim`) translates values and decides no application behaviour.
4. The page (`web/`) draws what the core returns and does not reproduce a core rule.
5. Run `cargo test` after each meaningful core change.
6. For each new behaviour test, break the behaviour it covers once, watch the test fail, then restore it. Note the failure message in `SECOND-ORDER.md`.
7. Keep `SECOND-ORDER.md` while you work: one row per assumption, surprise, divergence from the plan, or item the person must do.
8. Deploying is the person's decision. Sam approved Pages deploys for this project in advance (BRIEF.md, Autonomy): a push to `main` deploys after the check passes.
9. Before handoff, run `./scripts/check.sh` and record the result in `HANDOFF.md`.

Write `PLAN.md` from the brief and stop until the person has read it, unless the brief says the plan is approved in advance. When a rule in `BUILDER-SPEC.md` seems to block the work, stop and say so in a worklist row; do not loosen a check to make it pass.

The minimum-price numbers live in `crates/core/src/market.rs`. Strings live in `data/`. Art is TikZ: `python3 tools/art.py` writes `web/art/` from the sources it keeps in `tools/art/`. `python3 scripts/break_tests.py` breaks each covered rule once and expects its test to fail.
