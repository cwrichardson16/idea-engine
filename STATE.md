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
- T3 skeptic round 1 BLOCK (2): conflict mid-raid overwrote newer cloud save; Anki events lost after a conflict. FIXED: conflict → clear applied/sending, take_cloud, refetch (waits while a raid is on), adopt whatever its time; fetch delays upload one gap so the save holds applied events; newer-rule now uses the loaded save's wall (Cloud.local_wall set in load). 69 tests incl. 2 new regression tests. Round 2 pending.
- T3 skeptic r2 BLOCK: token expiry → refetch adopted own older upload. FIXED 2a50c7c (after first sync only a higher cloud version replaces; test). 70 tests. Round 3 pending.
- Chrome 2026-11-28 01:20: NO requests reached Supabase (DB empty, no cred in localStorage) — dev server had been started before app.json gained game.data; restarted (task bg0bi7lt3). Re-check: play → players/saves rows appear; localStorage exact.state.cred set; cloud chip shows.
- Seen at a 614 px wide window: HUD chips overlap (top bar wraps over the study panel and dev row; Staff button under Raid) → T5 phone layout must fix.
- T3 skeptic r3: VERDICT SHIP (2a50c7c). TODO small hardening after the dev build: Fetch step waits while fight::active always (not only take_cloud). Known accepted: lost save reply → ≤15 s of play lost on the following conflict.
- T3 cloud save VERIFIED in Chrome 2026-11-28 01:25 (dev server restarted so game.data loads): Play → join made 1 player, sign-in, upload; "☁ saved" chip; localStorage has cred; after Dev army + a review the cloud row is version 4 with interns 24 and reviews_today 52 = the screen. Screens .../screenshot-1795854282238-87.jpg, -1795854320874-88.jpg. Hardening: Fetch never runs mid-raid.
- T3 remaining: restore code (edge function: show a 6-char code; claim on another device moves rows; salted hash; 5 tries/hour; signs old account out) + bug report button + test web. Then T4 Anki.
- T3 restore code BUILT 2026-11-28 01:45: migration "restore" applied (restore_codes salted sha256, restore_attempts; RLS on, no policies), edge fn `restore` deployed (verify_jwt=false; make needs bearer; claim 5 tries/hour/IP, code deleted on use, new password signs old device out). curl proof: same account, old password refused, get_state on new device returns the save, 6th try → 429; test user + attempts deleted. Game: connector kinds mkcode/claim, cloud.rs ask_code/claim/restore_answer (claim → new cred, take_cloud refetch), commands `restorecode`/`restore <code>`, HUD restore_code, Account panel (tap the ☁ chip). 72 tests. NEXT: Chrome check of the panel + a second-profile claim, skeptic on the restore diff, then bug-report button, test web, T4.
- Restore skeptic r1 BLOCK: old device → sign-in fails → Join → uploaded a COPY of the moved village. FIXED: players.moved_at + put_save refuses tokens issued before the move (403 → auth); client sign-in failure → Step::Moved ("this village moved to another device"), no Join/upload; claim still possible. Live curl proof: old login save after move → 403, new device save ok, old password refused; test users cleaned. 73 tests. Restore skeptic round 2 pending.
- Conner 2026-11-28 ~01:50: "no coffee" → wants Coffee ☕ resource renamed; asked for more ideas (gave Supplies/Saline/Oxygen/Blood/Sleep/Sanity/Snacks/Pizza/Research/Credits). WAITING on his pick; then rename everywhere (economy names, contract text, tests, grep test).
- Restore skeptic r2: only blocker was the repo migration 003 missing the put_save body → fixed (full function in file, matches live). Minor fixed: 'Get my restore code' on a moved device now explains instead of hanging. 73 tests. Round 3 pending.
- Restore skeptic r3: SHIP. Live put_save verified to contain the moved check + row lock (pg_get_functiondef). NEXT: Chrome two-device restore (127.0.0.1:8765 = device A, localhost:8765 = device B, separate storage).
- Restore VERIFIED in Chrome 2026-11-28 06:41: device A (127.0.0.1) Account panel → code shown (screenshot-1795873268852-89.jpg); device B (localhost, fresh Hospital 1) typed "t7n6-rd6f" → loaded A's Hospital 2 / 2000 / 14 Knowledge / 24 staff (screenshot-1795873319558-90.jpg); A then changed (TH3) → "this village moved to another device", DB keeps v5 Hospital 2 save (moved_at set). Fixed: code input was invisible (dark on dark) → white box.
- T7 NOTE: a device that claims a code leaves its own starter account (players row + starter save) behind → matchmaking must skip untouched/abandoned accounts (or claim deletes the claimer's previous starter account).
- STILL OPEN T3: bug report button; `test web`. Rename Coffee waits on Conner's pick.
- Bug report BUILT + live-proven 06:52: insert 201, row tied to the sender with the save attached; test user deleted. 74 tests. NAMES.md written (helper's recommended set: Scrub In; Amber/Serum/Lumens; Amphitheater; The Blight faction). Waiting on Conner's naming direction. NEXT: test web (stop dev server first), then T4 Anki.
- test web 06:58: 5 passed, 0 failed (web build app.wasm 1.5 MB gzip). T3 COMPLETE (cloud save, restore, bug report; skeptic SHIP x2; Chrome + DB evidence). NEXT: T4 Anki — server first: pairings table + edge fn 'reviews' (code hash auth, ≥1.5s counted client-side, reject repeat last_id, cap 1500/day + rolling 24h, 30-day baseline on pair), then game: 'Link Anki' shows a pairing code (mkpair via edge fn with bearer), then Python add-on in village/anki-addon/ + a script that posts like the add-on (proof).
- T4 server LIVE 07:05: migration 'anki' (pairings, day_totals, credit_day service-only, log_reviews honour ≤200, max-not-sum, 1500/day + rolling 24h), edge fn 'anki' (make/pair/reviews; wrong codes 10/h/IP). Python add-on-style test PASSED: pair ok, 60 counted, repeat refused, cumulative 100 → +40, 9999 → capped total 1500, 30-days-ago refused, honour 200 → +0, 500 refused, get_state shows 1500. NEXT: game side (Account panel 'Link Anki' → mkpair code; honour button 'log today's reviews' → log_reviews), then the Python add-on in village/anki-addon/.
- Conner 2026-11-28 07:05 DECISIONS: (1) GO GENERIC theme (not med). Draft set (Gold/Ink/Crystals/Knowledge, Academy, Workshop, Mint/Treasury, Ink Press/Ink Cellar, Training Hall, Barracks, Ballista, Mage Tower, Recruit/Ranger/Golem) NOT yet approved — he asked for a WIDER name research scan (helper running → NAMES-WIDE.md). (2) NO honour/phone button: remove log_reviews + honour from day_totals path; SPEC-3 T4 phone fallback dropped. (3) Anti-cheat must be stronger: pressure-test a better design (helper running → ANTICHEAT.md) before building the add-on. (4) He questioned the computer "enemy" camps: core is Clash PvP. Told him: PvP raids (T7) are the core; camps only as tutorial target + backfill when no player is raidable. Await his OK.
- Test project has ~5 leftover test accounts (Chrome devices A/B etc.) → wipe before testers.
- 07:15 honour button DROPPED live (migration drop_honour). NAMES-WIDE.md done (rec: Stonehaven; Gold/Grain/Crowns; Keep; Mason's Lodge; Mint/Treasury; Farm/Granary; Barracks; Muster Ground; Ballista/Watchtower/Trebuchet; Spearman/Bowman/Lancer/Ram; Truce shield; leagues Yeoman→Sovereign) — awaiting Conner. ANTICHEAT.md done (rec 'review ledger': per-review records, server checks schedule consistency, 50 new/day, timely arrival, cap ramps 300→1500 over ~24 days, flags halve until reviewed, one-time pairing → device key; ~3–4 days). Adversarial review of ANTICHEAT.md running.
- 07:20 ANTICHEAT round 1: skeptic BLOCK (72h window kills phone-synced reviews; long-interval cards never credit; FSRS reschedule breaks spacing; cheap farm via forged 1-day-interval snapshot; UTC day). All 7 fixes accepted → designer revising ANTICHEAT.md; then skeptic round 2 (same reviewer, resume).
- 07:25 Conner approved the Stonehaven kingdom set ('sure i like those names'). Rename helper running (strings only; checks test-rust, contract [], grep). Camps renamed Bandit Camp/Raider Outpost/Rebel Fort/Warlord's Citadel pending his call on keeping camps (tutorial + backup only).
- 07:30 RENAME DONE (Stonehaven set): checks re-run by me: contract [], 74 tests pass, grep for med names empty; app.json title Stonehaven. Chrome look pending (dev server restarting).
- 07:35 ANTICHEAT pressure test DONE: skeptic r2 SHIP; amendments (backfill queue, growth rule Good/Easy only) written into ANTICHEAT.md. Build estimate ~4 days. NEXT T4 build per ANTICHEAT.md: migration 006 (review rows table, pairing device key, snapshot), edge fn rework, add-on in village/anki-addon/, game 'Link Anki' panel.
- 07:40 Rename VERIFIED in Chrome: tab 'Stonehaven', HUD 🪙 Gold / 🌾 Grain / 👑 Crowns / 📚 / 🔨 2/2 / 🏰 Keep 3, Army panel 'Spearman ×24 · Muster Ground full', 'Lancer · Needs Barracks level 2', ⚔ Raid (screenshot-1795875441911-92.jpg). Leftover: dev-only aria 'Dev: Town Hall up one' (testers never see).
- 07:45 Conner asked to pressure-test the path to 'good art + testers playing a robust CoC-like game'. ROADMAP-DRAFT.md written; 2 reviewers running in parallel (gameplay → ROADMAP-REVIEW-GAMEPLAY.md, art → ROADMAP-REVIEW-ART.md). Then merge into ROADMAP.md, report to Conner. T4 build waits behind this.
- 07:55 Gameplay roadmap review: agree-with-changes (camps don't refill/scale, mines cap 50–300 s, no HP scaling → easy 3-stars, no HP bars/shots, no move-buildings, PvP not built). Revised order ~18–22 agent-days: econ+combat fixes → readable battles → Anki (timebox) → move buildings → lean PvP + refilling camps → Trebuchet → tutorial/phone → art (timebox) → walls (stretch) → link/kit. Cuts: leagues, obstacles, streak builder, replays, Ram unless walls. Art review pending.
- 08:00 ROADMAP.md agreed (merged both reviews): 1 econ+combat fixes → 2 art spike (KayKit) → 3 readable battles → 4 Anki → 5 move buildings → 6 lean PvP + refilling camps → 7 Trebuchet → 8 full art (KayKit, 6–7 d) → 9 tutorial/phone → 10 walls (stretch) → 11 link/kit. ~22–27 agent-days. NEXT: step 1 unless Conner reorders.
- 08:05 Conner: pressure-test ROADMAP + goal/loop once more; present when reviewers agree; build only after his OK. GOAL-LOOP.md written. Round-2 reviewers running (product → ROADMAP-REVIEW-2-PRODUCT.md, tech → ROADMAP-REVIEW-2-TECH.md). Do NOT start /loop until Conner approves.
- 08:15 Roadmap r2 PRODUCT: BLOCK (5): no daily-return tracking (visits table + playtest_daily view); baseline vs capped credit mismatch (store raw filtered counts); per-step done-checks must be written + reviewed now; survey screen + erase-my-data + pre-launch wipe + tester list; real iPhone Safari check (no WebGPU fallback). Should-fix: 800 chest unreachable (credit ≤650 in 7d), add-on packaging, playtest-run phase, recruiting, measurable load/art checks, step 11 ≈2.5 d. Waiting on TECH review, then revise both docs.
