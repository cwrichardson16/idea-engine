# SPEC-3: Ready for testers (draft v1, 2026-11-28, to be pressure tested)

## Goal
A med student Conner trusts opens a private link on a phone or laptop and, with no help from Conner:
1. Starts playing straight away. The village is saved in the cloud and can be picked up on another device with a code.
2. Learns the loop from a short tutorial: build → collect → train → raid.
3. Plays the Clash-like core with med-school names. Dev buttons are hidden.
4. Studies their own Anki cards. Reviews earn Knowledge, the daily chest, a streak and a power hour.
5. Raids other testers' villages, gets raided back, gets a shield, and can take revenge.
6. Can send Conner a bug report. Conner can see in a table who came back each day and how many reviews they did.

Branch: claude/eloquent-goodall-rrq9xv. Never main.
Approved by Conner, all $0 only:
- one free Supabase project;
- a private link on a free host;
- free Canva/Figma images.
No paid APIs.

## Finish line (all must hold, with evidence in STATE.md)
- `bun exact.mjs test-rust` green.
- Contract builds to `[]`.
- `bun exact.mjs test web` green, with page tests for the tutorial, a study reward, a cloud raid and the result.
- A **deployed link** opened in Chrome, at desktop size and at phone size (390×844):
  - a fresh browser profile plays the tutorial;
  - it raids a second test account's village;
  - the defender sees "you were raided" and gets a shield;
  - a reload keeps everything;
  - the restore code works in a second profile.
- Anki: the add-on is installed in a local Anki copy, or a test script stands in for it. N reviews make Knowledge go up by the rule for N in the deployed game.
- Supabase:
  - `get_advisors` (security) shows no RLS errors;
  - a SQL test signed in as user B cannot read or overwrite user A's save, stats or raid log.
- skeptic + verifier both pass on the deployed flow.

## What a tester must NOT see
- "Clash", Gold/Elixir/Gems/Gold Mine/Barbarian, a red cross on white, or "Anki" in our name or logo. "Works with Anki" is OK.
- Dev buttons, debug overlays, raw error text or stack traces.
- Another tester's email or any private data. Opponents show only a display name and their village.
- Lost progress: a save that goes missing, troops vanishing, or loot taken twice.
- A paywall. No purchases at all in the test build.

## Feature checklist (build in this order, one at a time; each = tests + Chrome + reviewer + commit)
**T1. Med-school reskin + dev flag.**
- Names table in one place (`names.rs` or Contract `fn`):
  - Gold → Grant Money 💵, Elixir → Coffee ☕, Gems → Scrubs Tokens (or similar), Knowledge 📚 stays;
  - Town Hall → Teaching Hospital, Builder Hut → Resident Lounge, Gold Mine → Grant Office, Collector → Coffee Cart, storages → Bank / Coffee Store, Barracks → Lecture Hall, Army Camp → On-Call Room, Cannon → Defibrillator, Archer Tower → Syringe Tower;
  - troops: Fighter → Intern, Archer → Pharmacist, Brute → Attending.
- Emblem: a green cross or heart-pulse.
- Dev buttons hidden unless the build sets a dev flag; how depends on what Exact supports (a build-time feature, or a URL `?dev=1` read through a source).
- Proof: grep test → no old names in `app.contract` or the HUD strings; a page test that dev buttons are hidden by default; a screenshot.

**T2. Study loop (local first, fake review button).**
- Knowledge rate: 1 per 10 reviews (tunable).
- Daily chest: the first 50 reviews of a day give a chest of resources, scaled by day of streak; day 7 is big.
- Streak: days in a row with at least 20 reviews.
- Power hour: 100 reviews in a session → 1 h of 2× production and training.
- Proof: sim tests on the review → reward rules (day boundaries, the streak breaking, no double chest, power hour expiry using the game clock), plus a page test.

**T3. Tutorial + phone layout.**
- 5 guided steps with an arrow or highlight: build a Grant Office → collect → build a Lecture Hall → train → raid the Free Clinic. It can be skipped, and only shows once.
- The HUD fits at 390×844 with no overlap. Tap targets ≥ 44 px.
- Proof: a page test of the tutorial path, and phone-size screenshots.

**T4. Cloud: free Supabase project.**
- Anonymous sign-in (no email needed), plus a 6-character restore code to claim the same account on another device.
- Tables (RLS on every one):
  - `players` (id = auth uid, display_name, level, trophies, shield_until, restore_code_hash);
  - `saves` (player_id, contract, text, updated_at);
  - `events` (player_id, kind, n, at) for playtest stats;
  - `bug_reports`.
- Cloud save is the source of truth: write on change (debounced), read on Play. The local save is the offline cache. The newest updated_at wins.
- Proof: SQL checks with two users' JWTs, `get_advisors`, and a reload in a second browser profile with the code.

**T5. Multiplayer raids (async).**
- Find an opponent: a random other player near your level, not shielded and not online-raiding. There are 4 computer camps as a fallback when there are too few testers.
- The raid uses a snapshot of the defender's village (layout + storage), converted with the existing `plans_from_town`.
- Loot comes out of the defender's storage, capped.
- Afterwards the defender gets a shield (e.g. 8 h after ≥ 40% destroyed), a "you were raided" log entry, and a Revenge button.
- Anti-cheat for the playtest is light. The client sends a drop log; a Supabase edge function replays it with the same battle rules, or the playtest takes the client's word and stores the log for later checks (decide in the pressure test).
- Proof: two-account Chrome test on the deployed link; SQL checks that loot moved once.

**T6. Anki link.**
- A small Anki desktop add-on (Python). The player pastes a pairing code shown in the game. After each review session it POSTs the review count (counts only, no card content) to a Supabase edge function. The function writes an `events` row, and the game reads the new reviews on its next sync.
- Phone-only reviewers (AnkiMobile/AnkiDroid can't run add-ons): reviews still count once they sync and later open desktop Anki? (Open question; see Risks.)
- Proof: run the add-on code against the deployed function with a script; Knowledge rises in the game.

**T7. Private test link.**
- A static web build on Cloudflare Pages or Vercel, as an unlisted URL. Each tester gets a short one-page "how to play + install the add-on" guide.
- The bug report button writes to `bug_reports`.
- Proof: the deployed URL loads on a phone-sized window and a desktop. The verifier runs the whole finish line against it.

## Out of scope (after the playtest)
Purchases or the study pass, clans/wars/leaderboards, the study-only hero, the full art swap (only a few cheap free images if they help), native apps, trademark check, real name.

## Choices made (Conner can overrule)
- Anonymous sign-in + restore code instead of email logins: no email setup, and Supabase's free email limit is tiny.
- One Supabase project and static hosting; no custom server.
- Computer camps stay as a fallback opponent.

## Premortem (each one becomes an early check)
1. **The game can't call Supabase from the web build** (engine networking, CORS, auth headers). → Spike first in T4: one round trip from the deployed build before building on it.
2. **Testers review on phones, where add-ons don't run**, so the study loop never fires. → Decide the fallback before T6 (see Risks).
3. **An RLS hole lets one tester change another's village or loot.** → Two-user SQL tests and an edge function for any cross-player write (raid loot), never a client write.
4. **Cold web builds (25–46 min) eat the night.** → Batch Rust changes; never edit while a cold build runs; one heavy job at a time.

## Risks / open questions for the pressure test
- Is a night enough? What gets cut first?
- Anki on phones: the AnkiWeb sync can't be read by a third party. Options: desktop add-on only; a built-in "log my reviews" honour button capped per day; or reading AnkiConnect on desktop.
- Does Exact's web host support what T4–T7 need (fetch with headers, reading a URL flag, storing a token)? Check `.exact/checkout/docs/contract-for-agents.md` ("Declared sources", "App TypeScript/Rust data module").
