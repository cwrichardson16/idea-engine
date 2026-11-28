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
Steps: (1) DONE retune (economy.rs, SPEC-2, tests; battle stats in SPEC-2 table still to apply in battle.rs: Fighter 55hp 9dps 1.5t/s r0.7, Archer 24/8/1.5/4.0, Brute 360/12/0.9/0.7; cannon r7.5 dmg 10/14/18 per 0.9s; tower r8.5 dmg 5/7/9 per 0.45s; battle 150 s = 4500 ticks); (2) DONE battle.rs + tests/battle.rs (10 pass; camps Free Clinic/Urgent Care/Community Hospital/Mega Medical Center; plans_from_town; draw units with small per-index offset to avoid blob); (3) DONE campaign memory (Town.stars/looted, record_raid caps loot, save sections 4-5, SaveContract 3); (4) DONE fight.rs (commands, deploy on tap, step, result) + 5 sim tests; (5) DONE in code (camp buildings ids 1000+, rubble, red border planes, troop capsules) — needs Chrome look; (6) DONE raid HUD + dev army + raid page test; (7) DONE 12 battle + 43 sim tests; (8) DONE Chrome loop 2026-11-28: dev army 24, raid Urgent Care, red-border refusal toast, 3 stars 100%, loot capped (gold 1750->2000, elixir 1050->1700), home: army 10/24, stars 3/3, reload keeps all. Screens ~/var/folders/_s/dk7yz8xd5fg6j8rbsq5_1rj40000gn/T/claude-chrome-screenshots-OsU5ko/screenshot-17958508*. Troops were tiny dots: doubled size (not re-checked); (9) skeptic VERDICT SHIP (no blocking); test web 4/4 pass; verifier pending; (10) push.

## Conner approvals (2026-11-28, for the overnight run)
- OK: create ONE free-tier Supabase project for testing (accounts, cloud save, raids, playtest stats). $0 only.
- OK: deploy a private/unlisted test link on a free host (Vercel or Cloudflare Pages). $0 only.
- OK: free image generation in Canva/Figma. No paid API costs, ever.
- Mac stays awake by itself while Claude Code works.
- Verifier (Phase B): PASS on all 8 checks; bug fixed: double-click on a camp dropped a troop (taps ignored first 10 ticks). Troops look like plain dots (art phase).

## NEXT GOAL (overnight): ready for testers — see DESIGN-DIRECTION.md picks
1. med-school reskin of names + emblem (no red cross), dev buttons behind a flag
2. study loop: Knowledge from reviews, daily chest + streaks, power hours (fake review button first)
3. tutorial + phone layout
4. Supabase (free project): auth, cloud save, playtest stats table
5. multiplayer raids: opponent near level, raid snapshot (plans_from_town), shields, revenge, server replay check
6. Anki add-on sending review counts to Supabase
7. private test link on a free host
Each step: tests + Chrome check + reviewer + commit/push on the feature branch. Never main.

## CURRENT GOAL: SPEC-3.md (ready for testers) — big-build loop, started 2026-11-28
- Conner 2026-11-28: "just keep building ... if you take longer than tonight thats fine ... dont change up our plan whatever us and adv rev think ... no go until i give final go".
  Meaning: build all of SPEC-3 (incl. approved free Supabase project + private link for our own verification), apply reviewer fixes, but DO NOT share the link with testers / call it launched until Conner gives the final go.
- Pressure test: 2 reviewers running (technical "will it work" + product "what it misses"). Apply confirmed fixes to SPEC-3, then build T1→T7.
- Pointer registered (set_pointer.py). Old HANDOFF moved to HANDOFF-done.md.
- 2026-11-28 00:50: SPEC-3 v2 pressure-tested: both reviewers VERDICT SHIP. Build order T0 depth+web-build spike → T1 reskin+dev gate → T2 study rules → T3 Supabase via Rust data crate (tennis pattern, put_save RPC) → T4 Anki add-on → T5 tutorial/phone → T6 private link + tester kit (NOT shared until Conner's final go) → T7 multiplayer.
- NEXT ACTION: T0 — read economy.rs tables (KINDS, costs, timers, production, storage, max_count, MAX_LEVEL=3); extend to level 8 with Town Hall gating; time-to-max sim test; then web-build spike (stop dev server first: task b42dkl2xa).
- T0 DONE (commit "T0: levels 1-8"): MAX_LEVEL 8, timers to 8 h, affordability + time-to-max test (56 tests). Release web build spike OK: `bun exact.mjs web-build --json` → village/target/web-dist (13 MB, app.wasm 1.5 MB gzip + gpu 0.8 MB gzip; trust "development" — check a release/trust flag before T6). `--out` is for JS apps only.
- T1 in code (commit "T1: med-school names"): names in economy.rs/fight.rs, HUD emojis 💵 ☕ 🩺 🧑‍⚕️ 🏥, working title "Grand Rounds"; devMode persist + 5 taps on hud-th; world DevMode resource refuses `dev`/`review` unless `devmode on`. test-rust 56 pass, contract []. Running test web; then Chrome look (dev server: `bun exact.mjs web --port 8765` from village/, background) + skeptic, then T2.
- Scratch: scratchpad/t1_reskin.py (applied).
- T1 test web 4/4 (dev hidden, names). T2 in code + committed: 61 rust tests; HUD study panel; tz from time.utcOffset (declared in shape Time); SAVE_CONTRACT 4. test web running (5 tests). Skeptic on T0-T2 next, then one Chrome look for T1+T2 together, then T3 (Supabase spike).
- T2 test web 5/5. Skeptic round 1 BLOCK: power-hour bank above holds*1000 made decode refuse the save → fresh village overwrote it. FIXED (valid allows holds*2000, reload test), plus "Need N Scrubs Tokens", aria labels coffee/grant money, settle_training before a new power window. 62 rust tests. Skeptic round 2 pending.
- T3 recipe saved in NET-RECIPE.md.
- T0-T2 skeptic round 2: VERDICT SHIP (df8e515).
- T3 STARTED: free Supabase project created 2026-11-28 ($0, Conner's only org, free plan): grand-rounds-playtest, ref lqcecndhhmbifrxbexne, us-west-1. It takes the org's 2nd free active-project slot — tell Conner.
- Use ONLY the plugin Supabase tools (mcp__plugin_supabase_supabase__*) with project_id lqcecndhhmbifrxbexne. The bare mcp__supabase__* tools point at a different existing project — never use them for this game.
- T3 server side DONE 2026-11-28 (files in village/supabase/): migration 001_init.sql applied (players, saves+version, events ledger, bug_reports, survey; RLS on all; put_save versioned RPC; get_state RPC). Security advisors: only the 2 expected "authenticated can run security-definer RPC" warnings, no RLS errors. Two-user SQL test: stale write refused, B can't read/insert A's save or grant itself events, A's save intact. Test users deleted.
- Anonymous sign-ins are OFF on the project (would need a dashboard toggle) → instead edge function `join` (verify_jwt=false, 60 joins/hour global cap) creates a user with a random @players.grandrounds.invalid email + random password and returns them once; the game keeps them (persist) and signs in with POST /auth/v1/token?grant_type=password, re-signing on 401 (no refresh-token handling needed).
- Curl round trip OK: join → token → get_state null → put_save v1 → stale refused (conflict + server text) → get_state returns save → direct insert into saves refused by RLS.
- Project URL https://lqcecndhhmbifrxbexne.supabase.co ; publishable key sb_publishable_INxedNXuIKyn3VJZxYpSgg_enBjwiaY (public by design).
- NEXT: game side — village/data crate per NET-RECIPE.md (source `supa`, grants net.fetch https://lqcecndhhmbifrxbexne.supabase.co), world puts numbered JSON requests in hud.ask; answer returned via #[live] net option. Flow: on Play → if no cred: join; sign in; get_state → load newest (cloud vs local by version/updated); every save change → put_save with base version; conflict → reload cloud.
- T1+T2 Chrome look 2026-11-28: title "Grand Rounds", names/emojis in HUD, dev hidden until 5 taps on Hospital badge, study panel 0→51 today, Knowledge 4→14, streak 1 day, chest 1 toast, next chest "at 200". Screens .../screenshot-1795853338185-83.jpg, -84, -85.jpg. Fixed: chest with full storage said "+0 +0" → now says storage is full. T1+T2 DONE.
- T3 game side in code: village/data crate (Supa: join/signin/state/save → one tab line), app.json game.data, logic/src/cloud.rs (Off→Join→SignIn→Fetch→Ready; versioned uploads only when the village changed, ≥5 s apart; conflict adopts the other device's save; offline retry 15 s; Anki review events applied once via Study.last_event), Options.net live, HUD ask/cloud, Contract cred persist + supa(hud.ask) + net=reply. 67 rust tests (2 data + 13 battle + 52 sim). contract []. NEXT: test web + real round trip from the dev web build in Chrome (watch the players/saves tables), then skeptic.
