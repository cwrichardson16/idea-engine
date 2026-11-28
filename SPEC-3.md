# SPEC-3: Ready for testers (v2 after 2 reviewers, 2026-11-28)

Conner 2026-11-28: build all of it, take as long as it needs, go with what we and the reviewers agree. Do NOT share the link with testers or call it launched until Conner's final go.
Branch: claude/eloquent-goodall-rrq9xv, never main.
Approved, all $0: one free Supabase project, a private link on a free host, free Canva/Figma images. No paid APIs.

## Goal
A med student Conner trusts opens a private link on a phone or laptop and, with no help:
1. Plays at once. The village is saved in the cloud and can be claimed on another device with a restore code.
2. Learns the loop from a short tutorial.
3. Plays the Clash-like core with med-school names and no dev buttons, and still has things to build on day 7.
4. Studies their own Anki cards. Real reviews earn Knowledge, the daily chest, a streak and power hours.
5. Raids computer camps, which refill daily, and other testers' villages. Gets raided, gets a shield, can take revenge.
6. Can report a bug.

Conner can see daily return, study minutes per day against each tester's own 30-day baseline (review counts alongside), and exit-survey answers (v3: minutes + queue cleared, owner decision 2026-11-28).

## Finish line (evidence in STATE.md)
- **Checks:** `test-rust` green, contract `[]`, `test web` green. Page tests cover the tutorial, a study reward, the raid result and hidden dev buttons.
- **Deployed link in Chrome**, at desktop size and at 390×844:
  - a fresh profile plays the tutorial;
  - raids a computer camp;
  - raids test account B;
  - B sees "you were raided" plus a shield and loot loss, applied exactly once;
  - a reload keeps everything;
  - the restore code works in a second profile.
- **Anki:** the add-on code is run against the deployed function with a script, or installed in a local Anki. Credited minutes raise Knowledge by the rule (1 per minute, each review capped at 30 s); queue cleared pays its chest only when every due card was reviewed. Reviews under 1.5 s and duplicate posts don't count (v3: minutes + queue cleared, owner decision 2026-11-28).
- **Supabase:**
  - security `get_advisors` is clean;
  - SQL as user B can't read or write A's save, events or pairing;
  - loot moves only through the raid RPC.
- **Reviews:** skeptic + verifier pass on the deployed flow.

## A tester must NOT see
- "Clash", Gold/Elixir/Gems/Gold Mine/Barbarian names, a red cross on white, the blue Star of Life, or "Anki" in our name or logo ("works with Anki" is OK).
- Dev buttons, the test Review button, debug overlays, raw errors.
- Other testers' private data. Opponents show only a chosen display name, never a real name by default.
- Lost or duplicated progress: a save that goes missing, loot taken twice, a server edit overwritten.
- Any purchase or paywall.

## Build order (each item: tests + Chrome + reviewer + commit; cut from the bottom if time runs out)
**T0. Content depth + release-build spike.**
- Max level 3 → 8 for every kind.
- Costs and timers grow from about 1 minute to about 8 hours by level 8. The Town Hall gates levels.
- Storage caps grow so upgrades stay affordable. New level stats are added to SPEC-2's numbers table.
- Spike: run `bun exact.mjs web-build --out dist` once, then `serve dist`, and note its size and that it works. This proves static hosting early.
- Proof: sim tests that every level is reachable and affordable within caps; the total time to max is about 2–3 weeks of play with 2 builders.

**T1. Med-school reskin + dev gate.**
- Names, in one place:
  - Gold → Grant Money 💵, Elixir → Coffee ☕, Gems → Scrubs Tokens 🩺 (earned only; no store in the test build), Knowledge 📚.
  - Town Hall → Teaching Hospital, Builder Hut → Resident Lounge, Gold Mine → Grant Office, Collector → Coffee Cart, storages → Grant Bank / Coffee Store, Barracks → Lecture Hall, Army Camp → On-Call Room, Cannon → Defibrillator, Archer Tower → Syringe Tower.
  - Troops: Intern, Pharmacist, Attending.
  - Camps: Free Clinic, Urgent Care, Community Hospital, Mega Medical Center.
- Emblem: a green cross or heart-pulse.
- Dev buttons and the test Review button sit behind `state devMode = false persist`, turned on by a hidden gesture (for example, 5 taps on the level badge).
- The world's `dev` and `review` commands are refused unless the world is in dev mode, which a `devmode on` command sets from Contract.
- Proof: a grep test (no old names in the HUD/contract text); a page test that dev buttons are hidden by default; a screenshot.

**T2. Study rules (local; the dev Review button feeds them).** v3: minutes + queue cleared (owner decision 2026-11-28). Full rules and anti-cheat in ANTICHEAT.md "v3"; this replaces the review-count rules.
- All amounts are counted on the review **day**, which rolls over at 4 am local time, like Anki.
- **Credited minutes:** each credited review (ledger rules: on schedule, interval growth on Good/Easy, new cards capped at 50 a day, no cram/filtered/manual/rescheduled rows) is worth min(its time, 30 s). Reviews under 1.5 s are worth 0. Daily credit is capped at 120 minutes, with a ramp for new players: 60 → 75 → 90 minutes over the first 2 study days, then +1.5 a study day to 120 (about 3 weeks).
- **Knowledge:** 1 per credited minute. 1 Knowledge skips 60 s of a timer. So 60 minutes of study ≈ 1 hour of skips.
- **Daily chest tiers:** at 10 / 30 / 60 / 90 credited minutes. Each tier pays resources scaled by Town Hall level and the streak day. Each tier pays once per day.
- **Queue cleared:** a day where at least 1 review card was due and every due review card (read from the cards table, so daily limits don't hide any) got a valid review that day. Suspending, burying, deleting, "set due date" or forgetting due cards doesn't count as clearing; the server checks the due list against its own schedule. Pays 1 bonus chest (the size of the 30-minute chest). Reviews above the 120-minute cap still clear cards.
- **Streak:** days in a row that are queue cleared OR have ≥ 10 credited minutes. A day with nothing due (and nothing due by the server's schedule) freezes the streak, at most 3 days in any 7. Day 7 gives a big chest and Scrubs Tokens. A missed day resets the streak.
- **Power hour:** every 30 credited minutes adds 1 h (up to 4 h a day) of 2× production, with mine capacity also doubled, and 2× training speed.
- Study is a boost, never a gate.
- Proof: sim tests for the day rollover, the 30 s per-card cap and 1.5 s floor, tiers paying once, the cap and ramp, queue cleared (cleared, one card left, nothing due), the streak continuing, freezing and breaking, power hour expiry on the game clock, and capacity during a power hour.

**T3. Cloud (free Supabase project) through a Rust data crate.**
- Copy the tennis sample's pattern: `village/data` crate, `game.data` in app.json, `net.fetch` grant, `bun exact.mjs lock`. Raw PostgREST/Auth calls in Rust.
- **Sign-in:** anonymous sign-in. The tokens live in `state … persist`, with a refresh step.
- **Restore code:** an edge function moves the old uid's rows to the new uid. The code is stored as a salted hash, with about 5 tries per hour.
- **Tables (RLS on all):**
  - `players` (id, display_name, level, shield_until, new_until);
  - `saves` (player_id, contract, text, updated_at);
  - `events` (player_id, kind, n, at);
  - `pairings` (player_id, code_hash, revoked);
  - `raids` (id, attacker, defender, gold, coffee, stars, percent, drop_log, applied, at);
  - `bug_reports`;
  - `survey`.
- **Save rule:** every save write goes through one security-definer RPC, `put_save(text, base_version, raid_ids[], event_ids[])`. In one transaction it:
  - updates `saves` only if `version = base_version`, then adds 1 to the version;
  - marks those ledger rows applied (only the caller's);
  - returns a conflict on a version mismatch. On a conflict the client reloads the cloud save and never overwrites.
  The client skips the write when nothing changed. Server effects arrive only as ledger rows (raids, reviews). The raid RPC reads the defender's storage from the `v2|ms|gold,elixir,…` head of the save text.
- **Day rollover:** Contract passes `time.utcOffset` to the world in `load` (and on change) for the 4 am rollover.
- The Anki edge function uses `verify_jwt = false`. The hashed pairing code is its authentication.
- **Spike first:** one round trip from the web build before building more.
- Proof: two-user SQL checks, advisors, a reload, the second-profile restore.

**T4. Anki link.** (v3: minutes + queue cleared, owner decision 2026-11-28). Built per ANTICHEAT.md Design B (review ledger) and its "v3" section; this replaces the old count post.
- A desktop add-on (Python):
  - Pairs with a single-use code shown in the game (expires in 10 min), swapped for a device token.
  - While reviewing (debounced) and after each sync, it sends per-review rows since the last one sent: hashed card id, type, ease, time, intervals. Never card or deck content.
  - With each batch it also sends today's due list (hashed ids of review cards due today, from the cards table) for the queue-cleared check.
- The edge function applies the ledger rules (schedule-consistent, growth on Good/Easy, 50 new cards a day, no type 3/4/5 credit, 1.5 s floor), sums credited minutes (each review capped at 30 s), caps a day at `min(120, ramp)` minutes, decides queue cleared, and writes `events` rows that the game applies (ledger rule).
- At pairing, the add-on also sends the last 30 days of revlog rows; the server keeps daily minutes and counts with the same filter as the tester's baseline (analysis only, never credit).
- No honour button (dropped by Conner; migration 006 removes it). Phone reviews count once desktop Anki syncs them into the log, with their original timestamps.
- Proof: a script posts like the add-on against the deployed function; the minutes values match ROADMAP step 4 exactly and every cheat case gets its stated outcome.

**T5. Tutorial + phone layout.**
- 5 guided steps: build a Grant Office → collect → Lecture Hall → train → raid the Free Clinic. It can be skipped and only shows once.
- Fits 390×844 with tap targets ≥ 44 px.
- Proof: a page test and phone screenshots.

**T6. Private link + tester kit.**
- Cloudflare Pages or Vercel serves the web-build output, with SPA fallback to `shell.html`.
- A one-page tester guide:
  - how to play;
  - how to install the add-on;
  - exactly what is collected (per-review times, buttons, intervals and hashed card ids, never card content);
  - how to erase your data.
- A bug report button. A 3-question exit survey.
- Success lines, set now:
  - at least 5 of 10 testers active on day 7;
  - median raw study minutes per day (same filter as the baseline) at or above each tester's own 30-day baseline median minutes; review counts and queue-cleared days reported alongside (v3: minutes + queue cleared, owner decision 2026-11-28).
- The verifier runs the whole finish line on the deployed URL. **The link is not shared until Conner's final go.**

**T7. Multiplayer raids (async).**
- **Opponents:** any other player not shielded, matched regardless of level (the pool is tiny), plus the camps.
- **New players** get a 48 h shield.
- **Snapshot:** the defender's latest save goes through `plans_from_town`. Loot is 10% of storage, capped.
- **Raid RPC** (security definer): checks the shield, caps loot, rate-limits, inserts a `raids` row and sets the defender's shield (8 h at ≥ 40%).
- **Defender's client** applies the row once and shows "you were raided" plus a Revenge button.
- **Anti-cheat:** the client's word is taken, loot is capped, and the drop log is stored. No server replay: the battle code depends on the engine.
- **Computer camps** refill their loot every day.
- Proof: a two-account Chrome test on the deployed link; SQL shows each raid was applied exactly once.

## Out of scope
Purchases or the study pass, clans/wars/leaderboards, the study-only hero, the full art swap, native apps, the trademark check, the final name.

## Premortem → checks
1. The web build can't reach Supabase (CORS, grants, headers). → T3 spike: one round trip from the release build.
2. Testers study on phones. → the log reading picks phone reviews up after a desktop sync (no honour button) (v3: minutes + queue cleared, owner decision 2026-11-28); the guide says so.
3. RLS holes, duplicated loot or stale-device overwrites. → two-user SQL tests; a stale `put_save` is refused (SQL test); loot only through the RPC; ledger rows applied once (a sim test plus a SQL check).
4. Cold builds eat the time. → batch Rust edits; no edits while a cold build runs; one heavy job at a time.
5. Content runs out in a day. → T0 depth, checked by a time-to-max test.

## Round-2 additions (product reviewer SHIP; should-fixes accepted)
- **Phone vs desktop reviews:** (v3: minutes + queue cleared, owner decision 2026-11-28) superseded: the honour button is gone; phone reviews arrive through the desktop revlog after sync and are credited on their own Anki day (late rows accepted, backfill budget in ANTICHEAT.md).
- **Last applied ids in the save:** the save text stores the last applied raid id and event id, and the client skips anything at or below them. The `applied` flag is just for SQL checks. This works alongside `put_save`'s versioning.
- **Raid loss floors at 0:** the defender's loss stops at zero, and the attacker keeps the capped loot. That small duplication is accepted for the playtest.
- **Anki function cap:** (v3: minutes + queue cleared, owner decision 2026-11-28) caps each player at 120 credited minutes per rolling 24 h of row timestamps, as well as per Anki day.
- **Restore:** it signs the old anonymous account out. Its saves are refused once moved.
- `saves` has `version int not null default 0`. RLS on `saves`: the owner can read but has no direct insert or update; writes go only through `put_save`. `raids.applied` and `events.applied` are likewise changed only by RPCs.

VERDICT: both reviewers SHIP (technical after 3 rounds, product after 2). Spec approved under Conner's "go with what we and the reviewers agree".
