# Goal + loop for the build (draft for pressure test)

## Goal (one line)
Ten testers, given a private link, play Stonehaven — a robust, good-looking Clash-like base builder where real Anki reviews boost them — for 7+ days with no help, and nothing breaks, cheats cheaply or loses progress.

## Finish line (all must be true, each with evidence in STATE.md)
1. ROADMAP.md steps 1–9 and 11 done (10 walls is stretch); each step's own checks below pass.
2. Checks: `bun exact.mjs test-rust` green, `contract build --json` = [], `bun exact.mjs test web` green (page tests cover tutorial, a study reward, a raid result, PvP raid, hidden dev buttons, Link Anki).
3. Deployed private link in Chrome at desktop and 390×844: fresh profile does the tutorial; raids a camp; raids test player B; B sees "you were raided" + shield + loot loss once; reload keeps everything; restore code works in a 2nd profile; art loads under a size budget (first play < 10 s on a normal connection).
4. Anki: a script that posts exactly like the add-on proves counted reviews give Knowledge by the rule and every ANTICHEAT.md cheat case is refused; the add-on runs in a real local Anki (or the owner installs it once).
5. Supabase: advisors clean; SQL as user B can't read/write A's save, events, pairing, raids; loot moves only via the raid RPC; each raid applied once.
6. Balance: a sim test of a 7-day progression (with and without study) shows no dead ends, no 1-day max-out, raids not trivially 3-star at equal level.
7. Reviews: skeptic + verifier SHIP on the deployed flow.
8. Owner gives the final go before the link is shared (never shared by the agent).

## Loop (how the build runs unattended)
- One feature per iteration, in ROADMAP.md order. Per feature: write the step's done-check first → build (I am the only writer; helpers read/review only) → test-rust + contract + targeted page test → Chrome look and/or DB query → skeptic (and verifier for UI/flows) → fix confirmed issues (max 3 review rounds, then stop and report) → commit + push branch → update STATE.md (evidence pointers) → next.
- Heavy jobs one at a time (test web, web-build, cold builds); dev server stopped during test web.
- Context: at the kit's warnings write STATE.md + stamped HANDOFF.md; auto-compaction + the session hook resume from them.
- Stop and ask the owner only for: anything public, money, production, a design choice that changes the game's feel (e.g. cutting a roadmap step), or after 2 failed fixes of the same problem.
- Pacing: run with `/loop` (self-paced) using the prompt "Continue the Stonehaven build: read STATE.md and HANDOFF.md, do the next ROADMAP.md step per GOAL-LOOP.md, stop at the owner-only items."; the wakeup is a long fallback (20–30 min) since tool completions re-invoke me.
- Daily report to the owner in plain English: done (with evidence), next, blocked on him.
