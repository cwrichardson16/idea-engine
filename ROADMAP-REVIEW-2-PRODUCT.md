# Roadmap review 2: product angle (fresh eyes, 2026-11-28)

Reviewed: ROADMAP.md and GOAL-LOOP.md, against SPEC-3.md, ANTICHEAT.md, ROADMAP-REVIEW-GAMEPLAY.md, ROADMAP-REVIEW-ART.md, STATE.md, the code in village/ and the engine docs in ~/Projects/exact.

The question: will this plan produce a playtest that answers Conner's real question, "do students study more and keep coming back?", and can the finish line be checked?

Short answer: the build plan is solid for "a robust, good-looking Clash-like game". It is weak at the part that answers the question. Nothing in the finish line measures daily return or reviews against baseline. The database can't record daily return today. The baseline comparison as designed would compare different things. The finish line has no real-phone check, even though the game has no fallback for browsers without WebGPU. And the loop lets the agent write its own pass/fail checks.

Tags: CONFIRMED = I read it at the file:line or ran the command. UNVERIFIED = not checked at the source. Only CONFIRMED items are BLOCKING.

## BLOCKING

### B1. Daily return can't be measured, and the finish line never asks for it
- CONFIRMED: GOAL-LOOP.md (all 22 lines read; `grep -i "baseline|survey|return|active"` finds nothing) has no item about measurement. SPEC-3.md's goal said "Conner can see daily return, reviews per day against each tester's own 30-day baseline, and exit-survey answers". That line was dropped.
- CONFIRMED: the schema keeps no visit history. `players` has no last-seen or visit columns (village/supabase/migrations/001_init.sql:4-11). `saves` is one row per player, and `updated_at` is overwritten on every write (001_init.sql:13-19, :85). So "which days did tester X open the game" can't be answered after the fact.
- Fix (about 0.5 day, do it with step 4 so it exists before anyone plays):
  - Add a `visits(player_id, day)` table, upserted once per Anki day (4 am local, the same day rule as economy.rs:62) inside `get_state` or `put_save`.
  - Add one SQL view, `playtest_daily`: per player per day, visited yes/no, raw reviews, credited reviews, baseline median.
  - New finish-line item: on seeded fake data (3 fake players, 7 days), the view returns the expected D1/D2/D7 activity and reviews compared with baseline. Then delete the seeded data.

### B2. "Reviews against baseline" would compare different numbers
- CONFIRMED: the baseline is the raw 30-day daily counts the add-on sends at pairing (005_anki.sql:11, `baseline jsonb`; anki/index.ts:78-84).
- CONFIRMED: the planned credit is capped at `ramp = 300 + 50 × review_days` (ANTICHEAT.md:100). Only reviews that grow an interval earn credit (ANTICHEAT.md:75). New cards are capped at 50 a day (ANTICHEAT.md:93).
- If the analysis uses credited events (the only review numbers the game stores today, as `events`), a real student's test-week numbers come out below their raw baseline even if they studied more. The test would wrongly say "no effect".
- Fix: the server keeps a raw daily count, using exactly the same filter as the baseline: same review types, same 1.5 s floor, all rows whether credited or not. The success line uses raw against raw. Write the filter once, in one function the add-on uses for both the baseline and the live rows. Add a test that one set of revlog rows gives the same count both ways.

### B3. The finish line points at per-step checks that don't exist, and the agent writes its own
- CONFIRMED: GOAL-LOOP.md:7 says "each step's own checks below pass". There are none below. ROADMAP.md has no checks column (ROADMAP.md:5-17).
- CONFIRMED: GOAL-LOOP.md:17 says "write the step's done-check first". That means the builder sets its own bar, one step at a time, right before building. That is the easiest way to declare done falsely.
- Fix: before the loop starts, write a done-check for each of the 11 steps into ROADMAP.md: a test name, a SQL query or a screenshot with what it must show. Have them reviewed once, along with this plan. The loop may add checks but may not weaken them without telling Conner.

### B4. Exit survey, "erase my data" and wiping test data aren't built or checked
- CONFIRMED: a `survey` table exists (001_init.sql:41-45, insert policy :57), but the HUD has no survey UI (`grep -n -i survey village/app.contract` finds nothing; only the bug report is there, app.contract:157-163, :244-246).
- CONFIRMED: SPEC-3.md T6 promises the tester guide will say "how to erase your data". No erase function exists. The edge functions are only anki, join and restore (`ls village/supabase/functions`), and the HUD has no erase action.
- CONFIRMED: a restore claim doesn't delete the claiming device's own starter account. restore/index.ts deletes only the restore code (:73). STATE.md also records about 5 leftover test accounts. Leftover and abandoned accounts would inflate "players" and pollute "5 of 10 active on day 7", and they would show up as PvP targets.
- Fix (about 1 day, part of step 11, which grows from 1 to about 2.5 days):
  - A survey panel that opens from day 7 (or from a link in the guide), with the 3 questions written now. At least one should ask directly: "Did you do more Anki reviews this week because of the game? (less / same / more)".
  - An "erase my data" button that deletes the player's rows and auth user. Live-proven: the row counts go to 0.
  - A pre-launch wipe script. Finish-line evidence: a query showing only Conner's own accounts are left.
  - A roster: a `testers` list (player_id, label such as T01-T10) so the metrics count the 10 real testers and nothing else. Restore claims update the roster.

### B5. No real-phone check, and the engine has no fallback without WebGPU
- CONFIRMED: Exact has no WebGL renderer. On a browser without WebGPU the page shows a status message instead of the game (~/Projects/exact/game/README.md:372-381, "No WebGL renderer exists").
- CONFIRMED: the finish line only checks "Chrome at desktop and 390×844" (GOAL-LOOP.md:9). That is desktop Chrome with a phone-sized window. It is not Safari on iOS, not a real phone GPU or memory limit, and not touch.
- UNVERIFIED: whether WebGPU is on by default in testers' iPhone Safari, and how well it works on mid-range Android Chrome. I didn't check either.
- Fix:
  - Finish-line item: Conner (or the agent, on any real device it can reach) opens the deployed link on a real iPhone in Safari and, if possible, a real Android phone. Pass = the tutorial plays, a raid runs and a reload keeps progress. Record the device and OS version.
  - Recruiting screener: "iPhone on iOS version X or newer, or recent Chrome", whatever the phone check proves.

## SHOULD FIX (not blocking)

1. The top daily chest can't be reached during a 7-day test. CONFIRMED by arithmetic: the tiers are 50/200/400/800 (economy.rs:23). The credit ramp is at most 300 + 50 × 7 = 650 by day 7 (ANTICHEAT.md:100). Heavy studiers will also see "I did 500, the game counted 300". For the playtest, start the ramp higher (for example 600) or cut the top tier to 600. Show "counted X of Y reviews, here's why" in the study panel. This trades cheat payoff for the hook, so it is Conner's call: one line to him.
2. Installing the Anki add-on isn't a step. Missing pieces:
   - packaging an `.ankiaddon` file served from the private link (AnkiWeb listing is public, so it needs Conner's go);
   - an install test on the current desktop Anki for Mac and Windows;
   - the pairing flow when the game is on a phone (the code is shown on the phone and typed into desktop Anki);
   - guide screenshots.
   GOAL-LOOP.md:10's "or the owner installs it once" should become "a real review in desktop Anki turns into Knowledge on the deployed link, and one AnkiMobile or AnkiDroid review counts after a desktop sync" (evidence: screenshot plus SQL). About +1 day on step 4.
3. No playtest-running phase. The goal line ("for 7+ days ... nothing breaks", GOAL-LOOP.md:4) can only be checked after the link is shared, but the finish line stops before that. Add a phase 2:
   - a daily metrics report from `playtest_daily` and `bug_reports`;
   - a hotfix rule: redeploying a link testers are using counts as a production deploy under CLAUDE.md, so ask Conner or get a standing OK;
   - a D2 check-in ("decide early", per ROADMAP-REVIEW-GAMEPLAY.md:82);
   - the day-7 survey prompt;
   - a final readout against the success lines.
   UNVERIFIED: whether the free Supabase plan keeps backups. If unsure, take a daily `pg_dump` into the repo's private scratch area during the test.
4. Recruiting and onboarding are Conner's job and missing from the plan. Write the screener now (desktop Anki that syncs daily, a phone that passes B5, roughly 100+ reviews a day so baseline exists). Write the invite text for Conner to send, and set up a group chat for his daily nudge. UNVERIFIED: December exams may distort baselines either way; record each tester's exam dates in the screener.
5. Make the success lines exact before testers start:
   - "active on day 7" = a `visits` row on the tester's 7th Anki day;
   - "reviews" = the raw count from B2;
   - testers with no add-on paired are reported separately, not as zero.
   With n = 10 and no control group this is a directional signal. Say so in the readout.
6. "First play < 10 s on a normal connection" (GOAL-LOOP.md:9) can't be measured as written. Use the art review's budget (ROADMAP-REVIEW-ART.md:78): a test that sums the baked asset sizes, plus a timed load in Chrome with Fast 4G and 4× CPU throttling.
7. "Good-looking" has no check. CONFIRMED: no art item in GOAL-LOOP.md. Adopt ROADMAP-REVIEW-ART.md:91 (a fixed screenshot set scored by a fresh reviewer against a style sheet), plus Conner's thumbs-up on those screenshots. His taste is the goal.
8. ROADMAP.md:25's risk checks never reached the finish line (CONFIRMED, `grep -i "tuning|target|empty"` on GOAL-LOOP.md finds nothing):
   - a test that with 10 players all shielded, each still has 3 or more raid targets with loot;
   - the tuning table can be changed without a release;
   - a level-matched battle gives 1-2 stars.
9. Timeboxes have no overrun rule. Add: at 1.5× a step's estimate, stop and report to Conner with "cut / continue / trim".
10. Effort: step 11 is about 2.5 days, not 1 (survey, erase, wipe, roster, metrics view, guide, add-on download). Step 4 is about 5 days with packaging and the real-Anki tests. A realistic total is about 25-30 agent-days, plus cold-build wall time.

## What serves the goal and should stay
Steps 1, 3, 4, 6 and 9 serve the question directly (a reason to return, the study hook, playable on phones). Steps 2 and 8 serve Conner's stated art goal. Step 10 is correctly a stretch. The order is fine. The only move needed is to put the measurement work (B1, B2) inside step 4, so baseline pairing and raw counts are designed together.

## What I checked
- Read ROADMAP.md, GOAL-LOOP.md and STATE.md in full, plus SPEC-3.md, the relevant parts of ANTICHEAT.md, and ROADMAP-REVIEW-GAMEPLAY.md:70-100.
- Code:
  - village/supabase/migrations/001_init.sql (tables, put_save) and 005_anki.sql (pairings.baseline);
  - supabase/functions/restore/index.ts and anki/index.ts;
  - `ls supabase/functions`;
  - `grep survey/erase` in app.contract and cloud.rs;
  - economy.rs:22-23 and :62 (chest tiers, day rollover).
- Engine: ~/Projects/exact/game/README.md:368-384 (behaviour without WebGPU).
- Not checked: iOS Safari or Android WebGPU support, Supabase free-plan backups, the add-on install on real Anki versions. No builds run, nothing edited except this file.

VERDICT: BLOCK. Five confirmed gaps (B1-B5). All are small, about 2.5-3 extra agent-days in total, and fixable inside the plan, but without them the playtest can't answer whether students study more and keep coming back.
