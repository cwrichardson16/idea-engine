# Goal + loop for the build (v2, 2026-11-28, takes review round 2)

## Goal (one line)
Ten testers, given a private link, play Stonehaven (a robust, good-looking Clash-like base builder where real Anki study boosts them) for 7+ days with no help, nothing breaks, cheats cheaply or loses progress, and Conner can see daily return, raw study minutes vs each tester's baseline (counts alongside), and survey answers (v3: minutes + queue cleared, owner decision 2026-11-28).

## Finish line for the build (all true, each with evidence in STATE.md)
1. ROADMAP.md steps 1–9 and 11 done (10 walls is stretch), each passing its written done-check in ROADMAP.md. Checks were fixed before the loop started; none weakened without telling Conner.
2. Checks: `bun exact.mjs test-rust` green, `contract build --json` = [], `bun exact.mjs test web` green (page tests cover tutorial, a study reward, "counted X of Y", a raid result, PvP raid, hidden dev buttons, Link Anki, survey, erase).
3. Deployed private link (release build):
   - Chrome at desktop and 390×844: fresh profile does the tutorial; raids a camp; raids test player B; B sees "you were raided" + shield + loot loss once within 90 s without reload; reload keeps everything; restore code works in a 2nd profile.
   - Real phones: Conner runs the tutorial, a raid and a reload on his iPhone (Safari) and one Android phone (Chrome). Device + OS recorded. If the iPhone can't run it, Conner has decided what to do (ROADMAP step 2).
   - Load: ≤ 10 s from navigation to a tappable village, Chrome cold cache, "Fast 4G" + 4× CPU; ASSETS ≤ 2 MB gzip.
4. Anki (v3: minutes + queue cleared, owner decision 2026-11-28): the add-on-identical script proves credited minutes give Knowledge by the rule (1 per minute, 30 s per-card cap, 1.5 s floor, 120-minute cap with ramp), queue cleared pays only when every due card was reviewed, and every ANTICHEAT.md cheat case (incl. the v3 queue-cleared table) gets its stated outcome; a real review in desktop Anki becomes Knowledge on the deployed link (screenshot + SQL); same-filter test green (baseline and live raw minutes and counts use one function).
5. Measurement: `playtest_daily` proven on seeded data (3 players × 7 days, matches the fixture), seed deleted; raw daily counts stored for every post.
6. Supabase: advisors clean; SQL as user B can't read/write A's save, events, pairing, raids; loot moves only via the raid RPC; each raid applied once; erase leaves 0 rows for the player; the wipe script leaves only Conner's accounts.
7. Save safety: `old_saves_decode`, `decode_clamps_not_refuses`, `failed_load_never_uploads` green.
8. Balance: `seven_day_sim` passes its numeric bars (ROADMAP step 1); `small_pool_has_targets` passes.
9. Art: the 12-shot set passes the fresh reviewer and has Conner's thumbs-up.
10. Reviews: skeptic + verifier SHIP on the deployed flow.
11. Ship kit ready: guide, invite text, screener, survey; Conner has his tester list.
12. Owner gives the final go before the link is shared (never shared by the agent). Then ROADMAP Phase P runs.

## Loop (how the build runs unattended)
- One step per iteration, in ROADMAP.md order. Per step: read its done-check in ROADMAP.md (add checks if useful, never weaken) → build (I am the only writer; helpers read/review only) → test-rust + contract + targeted page test → Chrome look and/or DB query → skeptic (and verifier for UI/flows) → fix confirmed issues (max 3 review rounds, then stop and report) → commit + push branch → update STATE.md (evidence pointers) → next.
- Overrun: at 1.5× a step's estimate, stop and report to Conner with "cut / continue / trim".
- Any change to a number that save decode reads follows the SAVE_CONTRACT rule (ROADMAP step 1).
- Heavy jobs one at a time (test web, web-build, cold builds); dev server stopped during test web.
- Context: at the kit's warnings write STATE.md + stamped HANDOFF.md; auto-compaction + the session hook resume from them.
- Stop and ask the owner only for: anything public, money, production (including any redeploy once testers have the link), a design choice that changes the game's feel (e.g. cutting a step or vetoing the chest retune), a failed iPhone check, after 2 failed fixes of the same problem, and these known owner items (ask up front, in one message): open the step-2 spike on iPhone + Android; install desktop Anki on this Mac and do one real review; install Blender or skip it; recruit the 10 testers.
- Pacing: run with `/loop` (self-paced) using the prompt "Continue the Stonehaven build: read STATE.md and HANDOFF.md, do the next ROADMAP.md step per GOAL-LOOP.md, stop at the owner-only items."; the wakeup is a long fallback (20–30 min) since tool completions re-invoke me.
- Daily report to the owner in plain English: done (with evidence), next, blocked on him. During Phase P the daily report is the metrics report.
