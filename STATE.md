# STATE: Slice 2, army and battles (slice 1 done)

## Slice 2 (SPEC-2.md, pressure-tested over 2 rounds, commit after a88dd52)
- Phase A: camera, look, army buildings, home defenses, save v2.
- Phase B: battle sim, campaign, battle and result screens.
- [x] G1 camera  - [~] G2 look (TH, hut, scenery done; mines, storages and new kinds still to do)  - [ ] G3 army buildings  - [ ] G4 defenses  - [ ] G5 battle sim  - [ ] G6 battle UI  - [ ] G7 save v2
- Art:
  - ~/Downloads/kenney_castle-kit.zip (CC0, 2.2 MB, 76 GLB models).
  - The Fantasy Town and Nature kit downloads didn't land; Chrome likely blocked repeat downloads.
- Done in Phase A so far (commit 5f9b5b9, test-rust 29/29):
  - G1 camera: the `CamView` resource; `steer()` handles drag (>12 px) to pan and wheel to zoom, clamped; `tile_at(view, p, &CamView)`; a picking test.
  - G2 models:
    - village/art has 10 Kenney castle GLBs; their texture paths were rewritten by `village/tools/fix_glb_uri.py` (run it on any new Kenney GLB).
    - village/app.json turns assets on.
    - `MODELS`/`pieces()`/`SCENERY` in lib.rs: Town Hall = tower-square + hex roof + flag; hut = hex base + roof; trees and rocks around the edge.
    - Tests needing a world save use `let mut sim: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();`. That proves the bake accepts the models.
  - The web dev server was restarted (task bn4m8xc30, log ~/Projects/idea-engine-web.log). A cold build with assets was in progress at 5:24 PM; wait for "plan ready".
- Next:
  1. DONE 2026-11-27, Chrome: the models render (Town Hall tower, trees, rocks), drag pans, the wheel zooms (scroll up zooms in), and a tap selects the TH after zooming. Screenshots: /var/folders/_s/dk7yz8xd5fg6j8rbsq5_1rj40000gn/T/claude-chrome-screenshots-OsU5ko/screenshot-17958257*. The web rebuild with assets took about 3 min.
  2. DONE in code (commits 2a06d38 and after; 36 tests): G3/G4/G7 rules, the army panel, dev buttons, a rewritten app.test.contract. STILL TO DO: Chrome check of the army panel and new models (catapult, ballista, barracks); `bun exact.mjs test web`.
     Was: append Barracks, ArmyCamp, Cannon, ArcherTower to the END of KINDS (numbers in SPEC-2). Add an Army (trained counts plus a training queue with started_at) to Town; finish training in settle/away; housing counts the queue; cancel refunds. Add dev commands `dev rich` and `dev th`. Models: cannon = siege-catapult, archer tower = hex base + ballista.
  3. G7 save v2: freeze a literal v1 string test first, bump SAVE_CONTRACT, rewrite the "v2 refused" test (sim.rs ~416).
  4. Rewrite app.test.contract; run `bun exact.mjs test web`.
  5. Run the verifier on Phase A, then commit and push. Then Phase B (battle.rs).
- Avoid editing Rust while a cold web build runs: the watcher rebuilds on each save.

# Slice 1, the village (done)

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
  1. Red ghost checked in Chrome (OK). The verifier passed F1, F2, F3, F5 and F6; F4 upgrades were out of reach (they need 1000 gold for the TH).
  2. The verifier found a bug: double-clicking Place spent a gem, because "Finish now" appeared under the cursor. Fixed: nothing is selected after placing, and a test covers it (28/28). Check it in Chrome after the rebuild.
  3. Next slice: a look pass (pan/zoom camera, CC0 Kenney/Quaternius models), then the army and battles. Clash rules research is summarized in V1.md decision V8 and the chat; the next slice needs barracks, army camp, cannon, archer tower, walls, stars and loot.
- Browser screenshots: /var/folders/_s/dk7yz8xd5fg6j8rbsq5_1rj40000gn/T/claude-chrome-screenshots-OsU5ko/ (temporary).

## Decisions
See SPEC.md "Choices made" and V1.md §4.

## Evidence log
- 2026-11-26: Exact's garden sample renders 3D on the M1 (EXACT.md, Mac check).

## Learned / gotchas
- A sample's lock can go stale. Fix it with `bun game/app/shells.mjs <game> --lock`.
- The first cold web build takes about 25 to 46 minutes. `test-rust` can trigger a long dev-loop rebuild; wait for "plan ready".

## Look pass 2 (Conner 2026-11-27: "more cartoon, Clash-like"; approved "pressure test it but then go")
- Plan: LOOK-PLAN.md (diamond camera, screen-space drag, grid only while placing, chunky HUD; KayKit swap after Phase B).
- Done, uncommitted: tools/add_outline.py ran on art/*.glb (ink outlines + rim), brighter sky/grass/sun, far fog, dirt rim. test-rust 36 pass.
- The red "save refused: assets are not ready" box is the dev hot-swap only (snapshot refused while models load). Not a player bug.
- DONE a72df70 (pushed): steps 1-4. Skeptic caught drag lag + camera framing; fixed (35° lens, 60 m, grab-point drag incl. one-tick drags). 38 tests. Chrome: diamond view, drag, zoom, place shows grid / hides after, barracks placed.
- Known: mines/storages/army camp are still plain boxes (no outlines) until the KayKit swap after Phase B. Dev hot-swap overlay logged as engine feedback (not sent).
- Next: Phase A step 2 army Chrome check (barracks now building), test web, Phase A verifier, then Phase B.

## Direction decided (2026-11-28) — see DESIGN-DIRECTION.md "Conner's picks"
Med school theme; vinyl-toy + chunky characters art; Clash-like core + all 5 study mechanics + standard engagement features. Before public: new name, retuned numbers, themed names, no red-cross emblem.
Build order: Phase B battles (numbers retuned off Clash's) → study loop (Knowledge, daily chest, streaks, power hours) → playtest → multiplayer/pass/clans → art.

## SESSION GOAL (set 2026-11-28, Conner: "set goal ... loop until you are there")
Phase B done end to end: train → Attack → pick 1 of 4 camps → deploy outside red border → fight → result (stars, %, loot) → home with loot, troops used. Numbers retuned off Clash. Camps stored in the same layout text as a player village (multiplayer-ready). test-rust + contract [] + test web + Chrome loop with screenshots + skeptic + verifier + push.
Multiplayer later on a free Supabase test project (ask Conner before creating it).
Steps: (1) DONE retune (economy.rs, SPEC-2, tests; battle stats in SPEC-2 table still to apply in battle.rs: Fighter 55hp 9dps 1.5t/s r0.7, Archer 24/8/1.5/4.0, Brute 360/12/0.9/0.7; cannon r7.5 dmg 10/14/18 per 0.9s; tower r8.5 dmg 5/7/9 per 0.45s; battle 150 s = 4500 ticks); (2) DONE battle.rs + tests/battle.rs (10 pass; camps Free Clinic/Urgent Care/Community Hospital/Mega Medical Center; plans_from_town; draw units with small per-index offset to avoid blob); (3) DONE campaign memory (Town.stars/looted, record_raid caps loot, save sections 4-5, SaveContract 3); (4) Fight resource + commands (attack/deploy/surrender/home); (5) battle drawing; (6) HUD: campaign, battle HUD, result; (7) tests/battle.rs G5; (8) Chrome loop; (9) reviews; (10) push.
