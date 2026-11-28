# Anti-cheat for the Anki link (T4)

Written 2026-11-28 for SPEC-3 T4. Read-only research and design: nothing built, nothing deployed.

Goal: real Anki reviews give in-game boosts (capped at 1,500 a day). Make claiming reviews that weren't done hard. No honour or self-report button.

## The honest baseline

Read this first. Everything the add-on reads (the Anki database, the clock, its own Python code) sits on the player's computer, and the player controls it. An Anki add-on ships as readable Python, so any secret, signing key or check in it can be read and copied. Anki has no attestation (nothing that proves "this row came from a real, unmodified Anki"). AnkiWeb has no public API, so there is no outside source of truth to compare against.

So no design can make cheating impossible. A determined programmer can always write a script that posts realistic-looking reviews. What we can do:

1. Make the cheap cheats (sharing a code, posting a big number, pressing Again over and over, cramming, "set due date", clock tricks, faked old history) fail outright.
2. Make the remaining cheats cost about as much effort as studying, and cap their payoff at "what a diligent real student gets".
3. Flag odd patterns for Conner to look at (10 testers, so a human glance works), and never ban automatically.

## Research findings

### Revlog (the review log)
- CONFIRMED: the columns are `id` (epoch milliseconds of the review), `cid` (card id), `usn` (sync sequence number), `ease` (button: 1 = Again ... 4 = Easy; learning cards use 1 to 3), `ivl` (new interval: positive = days, negative = seconds), `lastIvl` (previous interval), `factor` (ease in permille, or FSRS difficulty scaled to 100 to 1100 when FSRS is on), `time` (milliseconds spent, up to 60,000), `type`. https://github.com/ankidroid/Anki-Android/wiki/Database-Structure
- CONFIRMED: in Anki's own code, `type` is Learning 0, Review 1, Relearning 2, Filtered 3 ("reviews before the due date or when rescheduling is disabled"), Manual 4, Rescheduled 5. `ease` 0 means manual rescheduling. The `id` is created from the current time in milliseconds. https://raw.githubusercontent.com/ankitects/anki/main/rslib/src/revlog/mod.rs
- CONFIRMED: after "reset" or "set due date", Card Info shows the type as Manual and the rating as 0. That means those actions leave a trace in the log. https://docs.ankiweb.net/stats.html
- CONFIRMED: the wiki says `lastIvl` is "not necessarily" the real gap between reviews. So comparing it to the gap has to allow some slack. https://github.com/ankidroid/Anki-Android/wiki/Database-Structure
- CONFIRMED: card ids are creation timestamps in milliseconds (same wiki). A raw card id gives away when a card was made. That's another reason to hash it.

### Sync and AnkiWeb
- CONFIRMED: syncing merges reviews from different devices, and both reviews of a card show up in its history. So phone reviews (AnkiDroid or AnkiMobile) land in the desktop revlog after the desktop syncs, with their original timestamps. A forced one-way sync can throw away one side's changes. https://docs.ankiweb.net/syncing.html
- CONFIRMED (absence): the AnkiWeb sync docs describe no third-party API. Anki does let you run your own sync server. https://docs.ankiweb.net/syncing.html , https://docs.ankiweb.net/sync-server.html
- CONFIRMED: the AnkiHabitica developer relies on the same "sync phone, then desktop reads the log" route for AnkiDroid reviews. https://github.com/NameLessGO/AnkiHabitica

### Add-on hooks
- CONFIRMED: `gui_hooks.reviewer_did_answer_card(reviewer, card, ease)`, `reviewer_will_answer_card(ease_tuple, reviewer, card)` (can change or skip a rating), `reviewer_did_show_question(card)`, `reviewer_did_show_answer(card)`, and `sync_did_finish()` all exist. https://raw.githubusercontent.com/ankitects/anki/main/qt/tools/genhooks_gui.py
- Consequence (my reasoning): any other add-on can change ratings through `reviewer_will_answer_card` or auto-answer cards. Our add-on can't tell whether a human pressed the button.

### FSRS
- CONFIRMED: FSRS keeps a memory state for each card: stability, difficulty and retrievability. Stability and difficulty change only when the card is reviewed. https://docs.ankiweb.net/faqs/what-spaced-repetition-algorithm.html
- CONFIRMED (forum): optimizing FSRS recomputes stability and difficulty for every card. So stored memory states can jump without any review happening, and the server must not treat a jump in stability as cheating. https://forums.ankiweb.net/t/what-is-the-revlog-ease-value-in-fsrs/54712
- UNVERIFIED: exactly how FSRS fuzz and "learn ahead" shift due times. I assume up to about ±5% of the interval, plus same-day early reviews of learning cards. The tolerances below allow for that.

### Prior art: leaderboards and Habitica
- CONFIRMED: the Anki Leaderboard (by Shige) developer has refused strict speed limits ("there are valid reasons why some people might review cards quickly") and suggests about 1 second per card as a reasonable floor. The support thread describes no server-side checks. https://forums.ankiweb.net/t/add-on-support-thread-anki-leaderboard-by-shige/51634?page=4
- UNVERIFIED (search snippet only, page not opened): in the same thread the developer said the leaderboard is open source and "the decks are just numbers", so faking data is easy. Users there also say auto-advance add-ons and "Grade Now" can inflate counts. https://forums.ankiweb.net/t/add-on-support-thread-anki-leaderboard-by-shige/51634
- CONFIRMED: an older leaderboard add-on reads the revlog and uploads only totals. https://forums.ankiweb.net/t/is-leaderboard-addon-safe/2504 (search summary; treat as UNVERIFIED in detail)
- CONFIRMED: AnkiHabitica scores on timebox or deck completion and does no verification. https://github.com/NameLessGO/AnkiHabitica , https://www.github.com/doc-bok/AnkiHabitica
- CONFIRMED: Review Heatmap is display only, with no server and no verification. https://github.com/glutanimate/review-heatmap
- UNVERIFIED: "Grade Now" (grading cards from the browser) arrived in Anki 25.07, and I don't know which revlog `type` and `time` it writes. https://forums.ankiweb.net/t/should-grade-now-be-usable-with-sm-2/64196 . The design below ignores rows with time under 1.5 s and refuses any type-1 review that comes too early, so it is covered either way. We should test it in a real Anki before launch.

Bottom line from prior art: nobody in the Anki world verifies reviews on a server. The leaderboards trust totals and live with cheating. So per-review checks against the schedule would be new ground. I found no off-the-shelf library to adopt.

## Holes in the current build (verified by reading the code)

File: `village/supabase/functions/anki/index.ts`, `village/supabase/migrations/005_anki.sql`.

1. The pairing code never stops working. It is sent on every request, so anyone who sees it (a screenshot, a shared add-on config) can post for that village. It also lasts forever until a new code replaces it.
2. `count` is trusted. Any number up to 1,500 is credited, and `last_id` only has to go up.
3. Every request checks the code against every pairing in turn (an O(players) loop). That's fine for 10 testers but won't scale.
4. `log_reviews` (the honour button) is still deployed. Conner dropped it, so it needs a revoke plus migration 006 (already noted in HANDOFF.md).
5. The baseline is whatever the client sends, and nothing is built on it. Fine, but it must never feed credit, because it can be faked.

## Three designs

All three share these basics:
- Device token instead of a reused code. The game shows a code that works once and expires after 10 minutes. The add-on swaps it for a random 256-bit device token, which the server stores only as a hash and looks up through an index. A new code revokes old tokens. Leaked codes then go stale within minutes, and the token sits only in the add-on's config.
- Privacy. Card content, deck names and field text are never sent. Card ids are sent as `HMAC-SHA256(salt, cid)`, cut to 16 bytes. The salt is made in the add-on and never leaves the computer, so the server cannot reverse it or match ids across players. If the salt is lost (reinstall), the player re-pairs and the server starts that player's card records again, and the trust ramp restarts too.

### Design A: "Better counts" (smallest change)
- What the add-on sends: `{token, day, count, last_id}`, as now. But the add-on drops type 3, 4 and 5 rows and time under 1.5 s, counts at most one credited review per card per day, and caps new cards at 50.
- Server checks: the device token, caps, the rolling 24 h cap, and `day` within ±1 of server time.
- How you beat it: edit the add-on's Python and post any number up to 1,500. Cost: 5 minutes.
- Build cost: about half a day.
- Verdict: stops outsiders only. The player is still trusted completely.

### Design B: "Review ledger" (per-review records, server checks the schedule)
What the add-on sends, as a batch every few minutes while reviewing (`reviewer_did_answer_card` triggers a send, debounced) and again after each sync (`sync_did_finish`, which picks up phone reviews):
```
{ token, reviews: [ { id, card: hmac16, type, ease, time_ms, ivl, last_ivl, factor } ... ] }
```
At pairing it also sends the last 30 days of the same records (the history snapshot), plus `{cards_total, cards_new}` counts. It never sends card or deck content.

What the server stores: `anki_cards(player, card_hash, origin [snapshot|new_counted|new_uncounted], last_id, last_ivl, last_type)` and `anki_reviews(player, id, card_hash, ..., received_at, credited, reason)`, keyed by `(player, id)`. Raw review rows are deleted after 60 days.

Server rules, applied to each row in id order. A row earns credit only if all of these pass:
1. Ordering and clock: the id is unique for the player. The id is no later than the server's time + 2 min, and no earlier than server time − 72 h. A row within 15 min of arrival is "live"; an older one is "late" (a phone sync). Anything outside that window is refused, which kills changing the system clock and backdating.
2. Type: only types 0, 1 and 2. Types 3 (filtered or cram), 4 (manual) and 5 (rescheduled) never count, and a type 4 or 5 row marks that card "tampered" until its next properly spaced review.
3. Time: `time_ms` must be at least 1,500.
4. The card must be known. It was either in the pairing snapshot, or its first-ever learning row (type 0, server never saw it before) was accepted into the new-card ledger: at most 50 a local day, first come, first served. Cards past the 50 are stored as `new_uncounted`, and none of their reviews ever earn credit. That stops "import 5,000 cards and farm them".
5. Spacing (the core check). For a type 1 review, the gap since the server's last row for that card must be at least `0.8 × last_ivl days − 12 h`. If the server holds no earlier row (a card from before the snapshot), the row's own `last_ivl` must be greater than 0. A review that comes early (because of set due date, a custom study session with rescheduling off, or a forged row) gets no credit.
6. Chain: the row's `last_ivl` should equal the server's stored `ivl` for that card. If it doesn't, the row still counts but the mismatch is logged (the wiki says these can differ, and FSRS optimizing can shift them). Cards whose chain is broken more than 5% of the time get flagged.
7. Per card per day: at most 1 credited type-1 row, 1 credited type-2 row (one lapse) and 2 credited type-0 rows. Pressing Again over and over earns nothing.

Daily credit = the number of credited rows, capped at `min(1500, ramp)`, where `ramp = 300 + 50 × live_days`. `live_days` counts the days on which the server saw at least 20 rows arrive live. That reaches the 1,500 cap after about 24 days of real use. The baseline the client sends is for analysis only.

Pattern flags (reviewed by Conner; a flagged day is credited at 50% and shows "checking" in the game; no automatic bans):
- Again rate on type 1 reviews under 2% across 300+ reviews in 7 days. (My assumption: real retention is about 80 to 95%, so 5 to 20% Again is normal. That fits FSRS default desired retention 0.9: UNVERIFIED, page not opened, https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-optimal-retention.)
- Metronome timing: time variation across a day under 0.25, or more than 40% of times inside one 200 ms band.
- More than 3 h of reviewing with no gap over 5 min.
- A day over 3× the player's server-observed 14-day median, once that median is above 50.
- Late rows (phone) making up over 70% of a week while the player claims to also use desktop. That one is info only.

How a cheater beats B, and what it costs:
- Fake poster: a script that simulates a scheduler and posts realistic rows in real time. It must keep a per-card schedule, respect intervals, fail about 10% of reviews, vary times like a human, and post live every day. Cost: a programmer with an AI can write it in an afternoon to a day. Payoff: capped by the ramp and by the 50-a-day new-card ledger, so it starts at 300 a day and builds like a real student would. This cannot be fully prevented.
- An auto-clicker in real Anki on a real deck: the revlog is genuine. The pattern flags catch naive bots (always Good, steady timing). A randomized bot beats them. Cost: a few hours of setup, and the computer stays busy. Payoff: still limited by how many cards are actually due, which only grows at the pace of the 50-a-day new-card ledger. Cannot be fully prevented.
- A trivial deck tapped honestly by a human (cards like "1+1"). Cost: about 2 to 4 s a card, so 1,500 reviews is 50 to 100 minutes of real tapping every day, while cards are still bounded by the due schedule. Can't be detected without seeing content. Accepted: it costs real time, about what studying costs.
- Forged history: write fake revlog rows straight into SQLite before pairing, to start with thousands of "mature" cards. Partly blocked: credit still starts at 300 a day under the ramp, and the snapshot gives no credit by itself.
- Someone else studies on your account: can't be detected. Accepted.

Build cost: about 3 to 4 days. Add-on 1 day; edge function, two tables and rules 1.5 days; tests (an attack script per cheat below, against a local Supabase) 1 day.

### Design C: "Our own sync server"
The player points Anki and AnkiDroid at a sync server we host (Anki supports this: https://docs.ankiweb.net/sync-server.html). We read their whole collection from the sync stream and run the B rules on it.
- Pros: it picks up phone reviews with no desktop add-on, and the data comes through Anki's own sync path.
- Cons: the server receives all card content (a big privacy cost, which breaks the "never content" promise). Players lose AnkiWeb, because you can only sync to one server. The rows are still made by the client, so a fake client can still post anything. Third-party sync servers break when Anki changes its protocol (CONFIRMED, docs warning). Whether AnkiMobile supports a custom server is UNVERIFIED.
- Build cost: 2+ weeks plus ongoing upkeep.
- Verdict: no stronger against a real cheater than B, worse on privacy, much more costly.

Simpler alternative considered: A plus a 1.5 s floor. It's not better because the player can still post any number in 5 minutes. B's per-row checks against the schedule are what make faking expensive.

## Attacking Design B as a cheater, and the fixes

| # | Attack | Outcome under the B rules above | Fix added |
|---|---|---|---|
| 1 | Post the same rows twice / replay | Unique `(player,id)`: refused | none needed |
| 2 | Leak or share pairing code | Single-use, 10 min: dead | none needed |
| 3 | Change system clock, post future or past rows | id window vs server time: refused | none needed |
| 4 | Spam Again on one card (learning steps every minute) | ≤ 2 learn + 1 relearn credited per card per day | none needed |
| 5 | Custom study / filtered deck cramming | type 3: ignored | none needed |
| 6 | Select 1,000 cards, "set due date: today", review them | Manual rows mark cards tampered; gap < 0.8×ivl: no credit | none needed |
| 7 | Forget/reset cards so they become new again | Same cid, server already knows the card: no new credit; reviews must respect spacing | Hole found: after a reset, `last_ivl` is 0, so the spacing check passes trivially. Fix: a tampered card earns no credit until it has a type-1 review 1+ days after the Manual row, and a tampered card's type-0 rows never earn new-card credit |
| 8 | Delete and reimport a deck (new cids) | Treated as new cards: 50/day ledger | none needed |
| 9 | Make 5 Anki profiles, all posting to one token | All share one ledger and one ramp | Hole: the profiles post in parallel, each with real "due" cards it imported long ago. Fix: one device token per profile is fine, but credit is per player, so the ramp and ledger still cap it. Also flag 2+ distinct "first-seen card id ranges" from one token (info) |
| 10 | Write forged rows straight into the revlog to make the spacing look right, then press keys | Rows must arrive live or within 72 h; forged mature history still starts at 300/day | Hole: late (phone) rows within 72 h can be forged in bulk at day end. Fix: late rows count only toward a separate cap of max(200, 1.2 × 14-day median of live rows), unless the player has no desktop live rows that week (a phone-first player still ramps the same way) |
| 11 | Edit the add-on to mark everything "live" | Live is judged by the server's arrival time, not a client flag | none needed |
| 12 | Time inflation (set `time_ms` = 4,000 on 0.3 s taps) | Not detectable per row | Partly fixed by the metronome flag (a constant fake time looks robotic); can't be fixed fully |
| 13 | Fake poster / smart bot (see above) | Capped at a diligent student's ramp | Fix: none possible. Limit what it wins: boosts above the player's own 14-day median give half value, so a cheater's extra hours buy little. This is a game design choice for Conner, flagged as optional |
| 14 | Use "Grade Now" to grade hundreds of cards at once (UNVERIFIED type) | If it writes time ≈ 0 → 1.5 s floor drops it. If type 1 but early → spacing refuses | Fix: add an attack test in a real Anki 25.x before launch to confirm |
| 15 | Wipe the salt and re-pair to reset flags | Card records reset, but the ramp also resets to 300 | none needed (a reset costs the cheater) |
| 16 | Flood the endpoint | Per token: max 1 batch every 20 s, ≤ 2,000 rows; wrong token: 10/h/IP (existing) | none needed |

Each of these becomes an attack script in the test, so the defenses are proven by running them rather than described. The proof should post as a cheater for cases 1 to 11, 14 and 15 and show 0 credited.

## Recommendation

Build Design B, the review ledger, with the fixes for rows 7, 9, 10 and 14 above. Also drop `log_reviews` and the honour column path (migration 006), and replace the reused pairing code with the single-use code and device token.

Why: it's the only option that turns "type a number" into "build and run a scheduler simulator or a bot every day", and even then the payoff is capped at what a diligent real student would earn. It keeps card content private, works with phone reviews after a desktop sync, and costs about 3 to 4 days.

What it still can't stop (say this plainly to testers and to Conner):
- A player who writes a convincing fake-review script, or runs a randomized auto-clicker on a real deck.
- A player who honestly taps through a trivial deck. That costs real time, but they learn nothing.
- Someone else doing the reviews.
- Any data faked on the player's own computer that stays within normal-looking patterns. The client can't be trusted, and no Anki or AnkiWeb source exists to check against.

The defenses are caps, a ramp, checks against the schedule, and human review of flags, not proof.
