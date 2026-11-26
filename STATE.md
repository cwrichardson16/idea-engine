# STATE: Slice 1, the village

Goal: see SPEC.md. Branch: claude/eloquent-goodall-rrq9xv. Exact SDK: ~/Projects/exact. Tool PATH: $HOME/.cargo/bin:$HOME/.bun/bin.

## Status
- [ ] F1 grid and camera
- [ ] F2 shop and placement
- [ ] F3 production and collect
- [ ] F4 builders and timers
- [ ] F5 save and offline
- [ ] F6 speed-ups

## Now
- The game is at `village/`, scaffolded with `bun ~/Projects/exact/game/new.mjs <abs path>`. The name "game" is refused.
- Written so far:
  - `village/logic/src/economy.rs`: pure rules, all functions take `now` in ms.
  - `village/logic/src/lib.rs`: draws the village, handles taps and HUD commands, keeps time.
  - `village/logic/tests/sim.rs`: 20 tests.
  - `village/app.contract`: the HUD, shop, selected-building panel and placing panel. `contract build` gives `[]`.
- Running checks from `village/`:
  - `bun exact.mjs test-rust` (the first run is cold and slow).
  - `bun exact.mjs contract build app.contract --json`.
  - `bun exact.mjs web` for the dev loop.
- Done 2026-11-26, commit 3dd2779:
  - test-rust passes 25/25, plus the lints.
  - Saving uses route B: Rust emits `save v1|...`, the Contract `state village persist` keeps it, and Play posts `load ...`.
- Now:
  - The web build is running: `bun exact.mjs web --port 8765`, logging to ~/Projects/idea-engine-web.log.
  - The skeptic review is running.
- Next:
  1. Screenshot F1 to F6 in Chrome, and test a reload.
  2. Fix the skeptic's findings.
  3. Tick the boxes.


## Decisions
See SPEC.md "Choices made" and V1.md §4.

## Evidence log
- 2026-11-26: Exact's garden sample renders 3D on the M1 (EXACT.md, Mac check).

## Learned / gotchas
- A sample's lock can go stale. Fix it with `bun game/app/shells.mjs <game> --lock`.
- The first cold web build takes about 25 minutes.
