# STATE: Slice 1, the village

Goal: see SPEC.md. Branch: claude/eloquent-goodall-rrq9xv. Exact SDK: ~/Projects/exact. Tool PATH: $HOME/.cargo/bin:$HOME/.bun/bin.

## Status
- [x] F1 grid and camera: Chrome 2026-11-26, HUD 500/1000 gold and elixir, 50 gems, TH1 in the centre.
- [x] F2 shop and placement: clicks land on the tile under the cursor; a blocked tile shows "Something is in the way" and greys out Place; elixir 500 to 350.
- [x] F3 production and collect: collect took gold from 500 to 537; the glow shows when the mine is ready.
- [~] F4 builders and timers: the hut takes builders 1/1 to 2/2 and they show 1/2 while busy; the first-build timer counts down. An upgrade timer is not yet seen in the browser (it needs 1000 gold for the TH); the tests cover it.
- [x] F5 save and offline: after a reload the 3 buildings and the counters came back; the mine filled while away (+100).
- [x] F6 speed-ups: Review took 📚 0 to 1; Finish with 📚 took it to 0 and the building was done; Finish with 💎 took gems 50 to 49 with 10 s left.

## Now
- Commits: bfa3496 (the skeptic's fixes), then the ghost fix: a blocked ghost hid inside the Town Hall. It now floats above, and a red or green footprint sits on the ground.
- Next:
  1. Check the red ghost in Chrome after the rebuild.
  2. Run a verifier for the UI flows.
  3. Commit and push.
- Browser screenshots: /var/folders/_s/dk7yz8xd5fg6j8rbsq5_1rj40000gn/T/claude-chrome-screenshots-OsU5ko/ (temporary).

## Decisions
See SPEC.md "Choices made" and V1.md §4.

## Evidence log
- 2026-11-26: Exact's garden sample renders 3D on the M1 (EXACT.md, Mac check).

## Learned / gotchas
- A sample's lock can go stale. Fix it with `bun game/app/shells.mjs <game> --lock`.
- The first cold web build takes about 25 to 46 minutes. `test-rust` can trigger a long dev-loop rebuild; wait for "plan ready".
