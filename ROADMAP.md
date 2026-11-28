# Roadmap: good art + testers playing a robust Clash-like game (v2, 2026-11-28)

v1 merged ROADMAP-DRAFT.md + the gameplay and art reviews. v2 takes every finding (blocking and should-fix) from ROADMAP-REVIEW-2-PRODUCT.md and ROADMAP-REVIEW-2-TECH.md. $0 budget. The link is never shared until Conner's final go.

The playtest must answer Conner's question: do students study more and keep coming back? So measurement, the survey, erase and data wipe are built before anyone plays, not after.

## Steps

| # | Step | Why | Agent-days |
|---|---|---|---|
| 1 | Economy + combat fixes (mines hold hours, building HP by level, troop levels, defenses retuned, camps scale and refill daily) + save safety + the balance sim with a fixed player strategy | Loop runs dry after session 1; raids are easy 3-stars; a retune today can wipe a village | 2–2.5 |
| 2 | Art spike: Keep, Mint, one animated KayKit troop, one defense through the release web build; size + load test; Conner opens it on his iPhone and one Android phone | Find engine, art and phone problems first | 1–1.5 |
| 3 | Readable battles: HP bars + visible shots only (hit/death animations move to step 8) | Players need to see what's happening; animations need the step-8 troops | 0.5 |
| 4 | Anki link per ANTICHEAT.md (v3: minutes + queue cleared, owner decision 2026-11-28) + measurement (visits, raw daily minutes and counts, playtest_daily view) + add-on packaging/install + periodic re-fetch + chest retune + "counted X of Y minutes" | The hook testers sign up for, and the numbers that answer the question | 5.5 |
| 5 | Move buildings | Basic base-building expectation | 0.5–1 |
| 6 | Lean async PvP with its server work (raids table, trophies, opponent list + top-10 RPCs, skip abandoned accounts, game handles 'raid' events), 10% loot, 8 h shield, revenge, defense log; camps stay the main raid supply | The Clash core | 4–5 |
| 7 | Trebuchet (splash defense) | Defense variety | 0.5 |
| 8 | Full art pass with ONE CC0 set (KayKit): tiered buildings from kit parts, ~5 animations per troop incl. hit/death, team colours, icons rendered from our models, Kenney CC0 sounds/particles | Coherent look, $0 | 6.5–7.5 |
| 9 | Tutorial + phone layout | Testers play with no help, on phones | 1.5–2 |
| 10 | Walls (stretch) + Ram if walls ship | Classic Clash, but costly | 2–3 |
| 11 | Ship kit: exit survey, "erase my data", pre-launch wipe script, tester roster, tester guide + invite + screener, verifier on the live link | Clean data, testers' rights; then Conner's final go | 2.5 |
| P | Run the playtest (7–8 calendar days, see below) | The goal can only be checked while testers play | ~0.5/day |

Total build ≈ 25–29 agent-days (27–32 with walls), plus cold-build wall time (25–46 min each), plus the 7–8 day playtest run.

Overrun rule: when a step reaches 1.5× its estimate, stop and report to Conner with "cut / continue / trim".

## Done-checks (fixed now; the builder may add checks but never weaken one without telling Conner)

Every step also needs: `bun exact.mjs test-rust` green, `contract build --json` = [], skeptic review with no confirmed issue left, commit + push to the branch, evidence pointer in STATE.md.

**Step 1: economy, combat, save safety, balance sim**
- Save safety (tests in test-rust):
  - `old_saves_decode`: frozen save strings from every past format (v1, army, campaign, study, current v4, and every new format added later) all decode; building count and x/z positions match the frozen expectation.
  - `decode_clamps_not_refuses`: a save with gold over the new cap, banked amount over `holds`, a missing or extra troop entry, an extra or missing camp star, and a level over `max_level` decodes, with each value clamped and vectors padded or truncated.
  - `failed_load_never_uploads`: a corrupt save string leaves the stored text untouched, shows an error, and no `put_save` is queued (dirty stays false for 10 simulated minutes).
- SAVE_CONTRACT rule (written into village/CLAUDE.md in this step): any change to a number that `valid()` or `decode` reads (caps, holds, housing, max_count, size, max_level, troop or camp count) must bump SAVE_CONTRACT, add a migration from the old version, and add the old format's frozen string to `old_saves_decode` before it ships. This applies during the test week too.
- Economy numbers (tests): every mine and collector holds between 2 h and 12 h of production at every level; `building_hp(kind, level)` at level 8 is at least 3× level 1; camps refill once per Anki day (4 am rollover test).
- Balance sim (test `seven_day_sim`), fixed strategy: the builder upgrades the cheapest eligible building, collects every 4 h, plays 3 sessions a day at 8 am / 1 pm / 8 pm; run with 0 and with 30 credited minutes a day (v3: 30 Knowledge, the 10 and 30 minute chests, the queue-cleared chest, 1 power hour). The attacker trains a full army and drops it in 3 equal groups on the side nearest the Keep. Pass bars:
  - Keep level: day 1 = 2–3, day 3 = 3–5, day 7 = 5–7; never max (8) by day 7.
  - With 30 credited minutes a day, at least 15% more total upgrade levels by day 7 than without.
  - No dead end: in every 24 h window there is an upgrade the player can afford or one finishing.
  - Level-matched raids (50 seeded fights per day-7 base): average stars between 1.0 and 2.0, 3-star rate ≤ 20%.
- Chrome: a save made before step 1 loads after it with the same buildings (screenshot).

**Step 2: art spike + real phones**
- `bun exact.mjs web-build --release` (production trust) succeeds.
- Test `asset_budget`: baked ASSETS ≤ 2 MB gzip total; animated troop clips stripped to ≤ 5.
- Load time ("first play"): release build on a static server, Chrome with cache disabled (cold cache), DevTools network "Fast 4G" and 4× CPU throttling, fresh profile. Measured from navigation to the village visible and a tap selecting the Keep: ≤ 10 s. Record the number.
- Chrome screenshot: Keep, Mint, the animated troop walking, one defense, all rendered.
- Real phones (Conner, about 5 min): opens the spike link on his iPhone in Safari and on one Android phone in Chrome. Reports for each: renders yes/no, fps (dev overlay), seconds to the village, OS version. Pass = renders on both, ≥ 20 fps, ≤ 15 s on his phone network.
- If the iPhone fails: the engine has no WebGL fallback and we don't build one. Stop and Conner decides: (a) Android/desktop-only testers, screener says so; (b) ask the Exact engine owner for a fallback and wait; (c) another route he picks.

**Step 3: readable battles**
- Test: for a fixed battle seed, stars, % and loot are identical before and after this step (damage timing stays in battle.rs).
- Page test: during a battle, every damaged troop and building shows an HP bar whose fill equals its battle.rs HP fraction (±5%); each ranged attack spawns a visible shot.
- Chrome screenshot at 390×844 mid-battle: at least 1 HP bar and 1 shot visible.

**Step 4: Anki link + measurement** (v3: minutes + queue cleared, owner decision 2026-11-28; rules in ANTICHEAT.md "v3")
- A script posting exactly like the add-on, against the deployed function (v3 fix after skeptic, owner decision 2026-11-28). Fixtures are realistic so no pattern flag fires: times cycle through 8, 10, 12, 14, 16, 20, 22, 24, 26, 28 s (mean 18 s); about 10% of review rows are Again (lapses on cards with interval ≥ 2 days, all credited); a 6-minute break after every 25 reviews; intervals grow on Good/Easy. The "ramp 120" player has 22 completed review days of 60 credited minutes each (7-day Again rate 10%).
  - Ramp 120: 100 credited reviews (the cycle above, 1,800 s) give exactly 30 Knowledge, the 10 and 30 minute chests, and 1 power hour.
  - Ramp 120: 100 credited reviews cycling 6, 10, 14, 18, 22, 26, 28, 45, 60, 90 s (30% at the cap, under the 40% idling bar) give exactly 35 Knowledge (2,140 credited s after the 30 s cap; 53 without it).
  - 100 reviews of 1,200 ms give 0 Knowledge.
  - Ramp 120: 500 credited reviews (the 18 s cycle, 9,000 s = 150 min, breaks as above) give exactly 120 Knowledge, all 4 chests and 4 power hours. The same post on a fresh pairing (ramp 60) gives exactly 60 Knowledge and 2 power hours.
  - Flagged cases (each must show the flag and 50% of the capped credit; flags don't stack): metronome, 100 reviews all 18,000 ms → 1,800 s → 15 Knowledge; idling, 100 reviews with raw times cycling 31 to 90 s (all at the cap) → 3,000 s → 25 Knowledge; low Again, 7 days of 50 reviews each (18 s cycle) with no Again → days 1 to 6 give 15 Knowledge each, day 7 is flagged and gives 7; no gap, ramp 120, 620 reviews (18 s cycle) with no break over 3 minutes (186 min) → capped at 120 → 60 Knowledge.
  - every case in ANTICHEAT.md's cheat table and its v3 queue-cleared table has a script case with the outcome the table states (refused, capped, not cleared or flagged).
- Queue cleared script cases: 20 due, all 20 reviewed → cleared + bonus chest; 20 due, 19 reviewed + 1 suspended after listing → not cleared; the due list omits 5 of 20 server-surely-due cards → not cleared + flag; "set due date" on a due card → not cleared; 1,500 due all reviewed with minutes capped at 120 → still cleared; nothing due on either side → no bonus chest, streak frozen (and a 4th frozen day in 7 breaks it); a day with 10 credited minutes but not cleared keeps the streak; a day-learning card cleared by a type 0 row counts; an over-limit new card or imported scheduled card reviewed on time does not block clearing; a filtered-deck card is judged by `odue`, and moving surely-due cards into a no-reschedule filtered deck over the allowance → not cleared; phone rows for 3 past days arriving in one late sync turn those days queue cleared and the recomputed streak extends through them (v3 fix after skeptic).
- Same-filter test: one Python function filters revlog rows for both the 30-day baseline and live posts (types 0, 1, 2; 1.5 s floor; seconds = min(time, 30 s); credited or not). A unit test on one fixture of revlog rows gets the same count AND the same seconds both ways.
- Server stores raw filtered daily numbers (`raw_daily(player_id, day, raw_reviews, raw_seconds, credited_reviews, credited_seconds, due_count, cleared)`) for every post, credited or not, next to credited events. The baseline has per-day counts and minutes.
- `visits(player_id, day)` table, upserted once per Anki day (4 am local) on `get_state`/`put_save`. `playtest_daily` view: per player per day, visited, raw minutes, raw reviews, credited minutes, queue cleared, baseline median minutes and baseline median reviews. Proven on seeded fake data: 3 fake players × 7 days; the view's D1/D2/D7 activity and raw-vs-baseline rows (minutes and counts) match a fixture file exactly; then the seed is deleted (query shows 0 seeded rows).
- Chest retune (default; Conner may veto in one line): tiers 10 / 30 / 60 / 90 credited minutes plus the queue-cleared chest; daily cap `min(120, ramp_min)` with `ramp_min = min(120, 60 + 15 × min(d, 2) + 1.5 × max(d − 2, 0))`, d = completed past review days (before today) with ≥ 10 credited minutes; today's study never raises today's ramp (60 on day 1 → 75 → 90 on day 3, 120 on day 23 after 22 review days). Test: sim player with 90+ credited minutes a day gets the top tier on day 3 and every day after, and a 120-minute player reaches the 120 cap on day 23 (after 22 review days), not before. ANTICHEAT.md updated to match (done in v3).
- Study panel shows "counted X of Y minutes", whether the queue is cleared (or how many due cards remain), and the reason for any gap (page test).
- Periodic re-fetch: every 60 s while not in a battle. Chrome: with the game open, a scripted post appears as Knowledge within 90 s with no reload.
- Packaging: a script builds `stonehaven.ankiaddon`, served from the private link (no AnkiWeb listing without Conner's go).
- Real Anki (Conner, once): installs desktop Anki on this Mac (new app, ask first) and the add-on; pairs using a code shown on a phone-sized game window; does 1 real review. Evidence: screenshot + SQL row in raw_daily and events, Knowledge up on the deployed link. If Conner has AnkiMobile/AnkiDroid: 1 phone review counts after a desktop sync. Windows install: checked only if Conner or a tester has a Windows PC; otherwise listed as untested in the guide.

**Step 5: move buildings**
- Tests: move to a free tile succeeds; to a blocked tile is refused; save round trip keeps the new x/z; a building mid-upgrade keeps its timer after a move.
- Chrome screenshots before/after a move, and after reload.

**Step 6: async PvP**
- Migrations: `raids` table, `players.trophies`, SECURITY DEFINER RPCs `list_opponents` (≤ 10 near level, excludes self, shielded, abandoned starter accounts and non-roster accounts once the roster exists) and `top10`; raid loot computed on the server from the save head (10% of storage, capped).
- Rust test: data/src/lib.rs passes `raid` events (today it drops everything but `review`); applying the same raid id twice changes loot once.
- SQL as user B: can read only display name + layout of opponents through the RPC, never A's save text, events, pairing or raids; loot changes only via the raid RPC.
- Test `small_pool_has_targets`: 10 players all shielded, each still has ≥ 3 targets with loot (camps count).
- Chrome, two profiles: A raids B; B sees "you were raided", an 8 h shield and the loot loss once, within 90 s without reload; revenge works; defense log lists the raid; top-10 shows both trophies.

**Step 7: trebuchet**
- Test: one shot damages ≥ 2 troops inside its splash radius and none outside; `seven_day_sim` still passes. Chrome screenshot.

**Step 8: art pass**
- Tool: glTF-Transform via node (present). Blender only if Conner installs it (owner item, ask once); icons are rendered in-engine otherwise.
- Tests: every kind × level 1–8 resolves to a real model; baked bounds fit the tile footprint ±10%; ASSETS ≤ 2 MB gzip; each troop ≤ 5 clips incl. hit and death; hit/death are cosmetic (step 3's fixed-seed result unchanged); grep finds no Clash/Supercell names.
- Load time re-measured the step-2 way: ≤ 10 s.
- Fixed screenshot set (village Keep 1, village Keep 7, battle mid-fight, result screen, study panel, shop; desktop + 390×844 = 12 shots) scored by a fresh reviewer against a one-page style sheet: no shot rated "fails". Then Conner's thumbs-up on the set.
- `CREDITS.md` lists every CC0 source.

**Step 9: tutorial + phone layout**
- Page test: a fresh profile finishes the tutorial with no dev buttons in ≤ 15 taps.
- Page test at 390×844: no HUD element overlaps another or leaves the screen; tap targets ≥ 44 px.
- Conner: runs the tutorial on his iPhone and the Android phone (screenshot each).

**Step 10: walls (stretch)**
- Tests: troops path around or attack walls when blocked; `seven_day_sim` still passes. Chrome screenshot.

**Step 11: ship kit**
- Exit survey panel, opens from day 7 or a guide link. 3 questions: (1) "Did you do more Anki reviews this week because of the game? less / same / more", (2) "Would you keep playing next week? yes / maybe / no", (3) "What made you open it, or stop opening it?" (free text). Page test: answers land in `survey`.
- "Erase my data" button: deletes the player's rows in every table and the auth user. Live proof: SQL row counts for that id = 0 in all tables.
- Restore claim deletes (or marks abandoned) the claiming device's starter account. Test + SQL.
- Pre-launch wipe script: removes test, abandoned and starter accounts. Evidence: query shows only Conner's own accounts left.
- `testers` roster (player_id, label T01–T10); restore claims update it; `playtest_daily` counts roster players only.
- Tester guide: supported phones/browsers (from step 2), add-on install with screenshots, pairing from a phone, phone reviews count after desktop sync, how to erase. Invite text and screener for Conner (below).
- Verifier on the live link runs GOAL-LOOP.md finish line 3.

## Conner's jobs (owner-only)
- Before or during step 2: open the spike on iPhone + Android (5 min). Decide if the iPhone fails.
- Step 4: OK to install desktop Anki on this Mac, then install the add-on and do 1 real review. Optional: 1 phone Anki review.
- Step 8: OK on Blender (or skip it); thumbs-up on the screenshot set.
- Recruiting (during steps 9–11): send the invite to 10 testers. Screener (written by the agent in step 11): desktop Anki that syncs daily; a phone that passed step 2 (or a laptop); about 20+ minutes of Anki a day for 30 days so a baseline exists (v3: minutes + queue cleared, owner decision 2026-11-28); exam dates in the test week. Set up a group chat for a daily nudge.
- Final go before the link is shared.

## Phase P: run the playtest (after Conner's go)
- Day 0: wipe script run, roster filled, each tester paired (raw baseline present) or listed as unpaired.
- Daily (agent): query `playtest_daily` and `bug_reports`; send Conner a plain-English report (who visited, raw and credited minutes vs baseline, raw reviews for context, queue-cleared days, new bugs) (v3: minutes + queue cleared, owner decision 2026-11-28); take a `pg_dump` backup to the private scratch area (free-plan backups unverified).
- Hotfix rule: redeploying the live link is a production deploy, so ask Conner (or use a standing OK he gives for bug fixes). Any rules/number change follows the SAVE_CONTRACT rule and passes `old_saves_decode` before redeploy.
- Day 2: check-in with Conner: anyone not returning, anything broken, decide early.
- Day 7: survey prompt. Day 8: final readout.
- Success lines (exact): active on day 7 = a `visits` row on the tester's 7th Anki day, target ≥ 5 of 10; study = raw filtered daily minutes (types 0/1/2, ≥ 1.5 s, each review capped at 30 s) vs the same filter's 30-day baseline median minutes, target median at or above baseline; credited minutes, raw review counts and queue-cleared days reported alongside (v3: minutes + queue cleared, owner decision 2026-11-28). Unpaired testers reported separately, not as zero. n = 10 with no control group: a directional signal, said so in the readout.

## Cut, limits, risks
- Cut for the test: leagues and trophy matchmaking, obstacles, streak-earned 3rd builder, visual replays, clans, spells.
- Accepted limits: chunky faceted low-poly style; the Lancer may become a foot Knight; no push notifications on web; a numbers change needs a redeploy (Exact tables are baked), so "tuning without a release" means a quick redeploy under the hotfix rule.
- Risks and their checks: empty raid list (`small_pool_has_targets`); week-1 balance (`seven_day_sim` bars); Anki friction (real-Anki check + guide); iPhone WebGPU (step 2 phone check, Conner's call if it fails); lost progress (save safety tests).

## Round 3 amendments (product reviewer SHIP)
- Visits and raw minutes/counts use the player's Anki day: the game sends its UTC offset with get_state/put_save (or a `p_tz` arg); a review counts on the day it was done (revlog timestamp), not the day it arrived.
- Baseline medians (minutes and counts) use all 30 days, including zero-study days (v3: minutes + queue cleared, owner decision 2026-11-28).
- Phone fps bar: if a mid-range Android is under 20 fps, first retry at the engine's lower quality setting before calling it a fail (the engine's starter runs ~16 fps on a Pixel 5). If no on-screen fps counter exists, step 2 adds one behind dev mode.
- Daily database backup needs the database password: a Conner item (or rely on the free plan's backups if confirmed).

## Round 3 amendments (technical reviewer SHIP)
- Phone smoothness: the game itself shows an fps number in dev mode (release builds strip engine overlays); otherwise judge by eye.
- Playtest backup: a daily JSON export via SQL (pg_dump isn't installed), unless Conner provides the database password.
- Step 6 test: a defender's loot loss floors at 0 when they already spent the gold.
- Step 6: opponents see only the base layout, so the server extracts the buildings section from the save (extra work, counted in step 6).

Status: both reviewers SHIP after 3 rounds (2026-11-28). Awaiting Conner's approval to start the loop.

## Later decision (after the playtest): free vs premium study limits
Conner's idea (2026-11-28): free tier limits how much study speeds you up, premium is unlimited; generous early so players get invested. Agent's view: don't paywall study credit (contradicts "study is a boost, never a gate; Knowledge never sold"); instead taper for everyone (e.g. full credit to 60 min, half after) and sell things around studying (3rd builder, cosmetics, study pass premium track, extra training slot, stats report). The playtest has no purchases; the daily report shows how often testers hit 60 / 90 / 120 credited minutes so the limit can be set from data.
