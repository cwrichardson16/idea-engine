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
- RESOLVED (adversarial review finding, not re-checked by me): "Grade Now" (grading from the browser, said to arrive in Anki 25.07) writes type 3 rows with an ease, which the design already ignores. https://forums.ankiweb.net/t/should-grade-now-be-usable-with-sm-2/64196 (version claim from the forum, UNVERIFIED). A quick check in a real Anki 25.x during the build is still cheap.

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

Revised 2026-11-28 after an adversarial review blocked the first version. Changes: late rows of any age are accepted; the ramp counts review days by row timestamp; the snapshot covers all non-new cards; spacing runs from the last row with ease > 0; type 5 no longer marks a card tampered; only reviews that grow an interval earn credit; the day is the player's Anki day.

What the add-on sends, as a batch every few minutes while reviewing (`reviewer_did_answer_card` triggers a send, debounced) and again after each sync (`sync_did_finish`, which picks up phone reviews with their original timestamps):
```
{ token, anki_day, rollover_hour, utc_offset_min,
  reviews: [ { id, card: hmac16, type, ease, time_ms, ivl, last_ivl, factor } ... ] }
```
At pairing it sends a snapshot of every non-new card from the `cards` table: `{ card: hmac16, type, queue, ivl, due_day }`, with `due_day` converted to an absolute Anki day. It also sends the last 30 days of revlog rows, used only for analysis. It never sends card or deck content.

What the server stores: `anki_cards(player, card_hash, origin [snapshot|new_counted|new_uncounted], last_scored_id, last_scored_ivl, expected_ivl, tampered)` and `anki_reviews(player, id, card_hash, ..., anki_day, received_at, credited, reason)`, keyed by `(player, id)`. Pairing also stores `paired_at` (server clock) and the player's `utc_offset_min` and `rollover_hour`. Raw review rows are deleted after 60 days.

Day: each row's Anki day = floor((id/1000 + utc_offset_min×60 − rollover_hour×3600) / 86400). The default rollover is 4 am local. The batch's `anki_day` must be within ±1 of the day the server computes from its own clock and the stored offset. The offset is fixed at pairing and may change at most once every 7 days (for travel). Otherwise a client could slide the day boundary to fit two days' caps into one.

Server rules, applied to each card's rows in id order. A row earns credit only if all of these pass:
1. Uniqueness and clock: the `(player, id)` pair must be new; duplicates are refused. The id must be no later than server time + 2 min (no future rows). It must also be no earlier than `paired_at`, since anything older is covered by the snapshot. Late rows of any age after pairing are accepted, because phone reviews reach the desktop revlog days later through sync.
2. Order within a card: if the server already holds a scored row for this card with a higher id, an older row arriving late for that same card is stored but not credited. The honest case (the same card reviewed on phone and desktop before a sync) is rare.
3. Type: only types 0, 1 and 2 earn credit. Type 3 (filtered or cram, including Grade Now) never counts. Type 4 (Manual: set due date, reset or forget) marks the card tampered. Type 5 (Rescheduled, e.g. FSRS "reschedule cards on change") does not mark the card tampered. It may shorten the card's `expected_ivl`, but not below 50% of the last scored interval.
4. Time: `time_ms` must be at least 1,500.
5. The card must be known. It was either in the pairing snapshot, or its first-ever learning row (type 0, card not seen before) was accepted into the new-card ledger: at most 50 per Anki day of the row's timestamp. Cards past the 50 are stored as `new_uncounted`, and none of their reviews ever earn credit. Non-new cards that show up after pairing without a type 0 row (for example, a deck imported with its scheduling) are also uncounted.
6. Spacing: for a type 1 row, the gap since the card's last row with ease > 0 must be at least `0.8 × expected_ivl days − 12 h`. Manual rows (ease 0) never reset the clock. For a snapshot card with no scored row yet, the review must fall on or after its snapshot `due_day` − 1.
7. Growth: this closes the 1-day farm. A type 1 row with ease 2, 3 or 4 earns credit only if its new `ivl` is at least 2 days, or is greater than its `last_ivl`. A Good/Easy review that leaves the card stuck at 1 day earns nothing and flags the card. A lapse (type 1, ease 1) earns credit only if the card's last scored interval was at least 2 days. Lapses can be at most 20% of a day's credited type 1 rows.
8. Tampered cards earn nothing until they have a type 1 review at least 1 day after the Manual row, with interval growth as in rule 7. A tampered card's type 0 rows never get new-card credit.
9. Per card per Anki day: at most 1 credited type 1 row, 1 credited type 2 row and 2 credited type 0 rows. Pressing Again over and over earns nothing.
10. Chain: the row's `last_ivl` should equal the server's last scored `ivl`. A mismatch is logged but not refused, because FSRS optimizing and rescheduling can change it. Cards whose chain breaks more than 5% of the time get flagged.

Daily credit = credited rows for that Anki day, capped at `min(1500, ramp)`. Also, no 24 h window of row timestamps may hold more than 1,500 credited rows. `ramp = 300 + 50 × review_days`. `review_days` counts the distinct Anki days since pairing, by row timestamp, that have at least 20 credited rows. It can never exceed the calendar days since `paired_at` on the server clock, so backfilling doesn't speed it up. The cap reaches 1,500 after about 24 days of study. The client's baseline is for analysis only.

Pattern flags (reviewed by Conner; a flagged day is credited at 50% and shows "checking" in the game; no automatic bans):
- Again rate on type 1 reviews under 2% across 300+ reviews in 7 days. (My assumption: real retention is about 80 to 95%, so 5 to 20% Again is normal. That fits FSRS default desired retention 0.9: UNVERIFIED, page not opened, https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-optimal-retention.)
- Metronome timing: time variation across a day under 0.25, or more than 40% of times inside one 200 ms band.
- More than 3 h of reviewing with no gap over 5 min.
- A day over 3× the player's 14-day median, once that median is above 50.
- More than 5% of a day's Good/Easy reviews leave a card stuck at 1 day.
- Backfill: one batch carries rows for 3 or more past Anki days on which the server had received nothing. Normal for a phone user who syncs weekly, so this is info only. It is also what a bulk forger looks like.
- A snapshot over 20,000 non-new cards, or a snapshot where more than 30% of cards fall due within the first 3 days (an implausibly front-loaded collection).

How a cheater beats B, and what it costs:
- Fake poster: a script that simulates a scheduler. It must make a per-card schedule consistent with the snapshot, grow intervals, fail about 10% of reviews and vary times like a human. Because late rows are now accepted, it can post a whole week at once instead of every day. Cost: an afternoon to a day for a programmer with an AI. Payoff: capped by the ramp, which follows server calendar days, and by the 50-a-day new-card ledger. This cannot be fully prevented.
- An auto-clicker in real Anki on a real deck: the revlog is genuine. The pattern flags catch naive bots; a randomized bot beats them. Payoff is limited by real due cards and the ramp. This cannot be fully prevented.
- A trivial deck tapped honestly by a human: costs real time (2 to 4 s a card), and growing intervals mean each card soon stops coming up, so they need an ever-larger deck. Can't be detected without seeing content. Accepted.
- Forged snapshot of a large, realistic mature collection, fed by the fake poster. Not closed. It gets the same payoff as a real heavy user once the ramp allows: 1,500 a day from about day 24.
- Someone else studies on your account: can't be detected. Accepted.

Build cost: about 4 days. Add-on 1 day (snapshot, day maths, batching); edge function, two tables and rules 2 days; tests (an attack script per cheat below, against a local Supabase) 1 day.

### Design C: "Our own sync server"
The player points Anki and AnkiDroid at a sync server we host (Anki supports this: https://docs.ankiweb.net/sync-server.html). We read their whole collection from the sync stream and run the B rules on it.
- Pros: it picks up phone reviews with no desktop add-on, and the data comes through Anki's own sync path.
- Cons: the server receives all card content (a big privacy cost, which breaks the "never content" promise). Players lose AnkiWeb, because you can only sync to one server. The rows are still made by the client, so a fake client can still post anything. Third-party sync servers break when Anki changes its protocol (CONFIRMED, docs warning). Whether AnkiMobile supports a custom server is UNVERIFIED.
- Build cost: 2+ weeks plus ongoing upkeep.
- Verdict: no stronger against a real cheater than B, worse on privacy, much more costly.

Simpler alternative considered: A plus a 1.5 s floor. It's not better because the player can still post any number in 5 minutes. B's per-row checks against the schedule are what make faking expensive.

## Attacking Design B as a cheater, and the fixes

Second pass, against the revised rules.

| # | Attack | Outcome under the revised B rules | Fix added |
|---|---|---|---|
| 1 | Post the same rows twice / replay | Unique `(player,id)`: refused | none needed |
| 2 | Leak or share pairing code | Single-use, 10 min: dead | none needed |
| 3 | Change the system clock to post future rows | id > server time + 2 min: refused | none needed |
| 4 | Backdate rows to before pairing | id < `paired_at`: refused | none needed |
| 5 | Spam Again on one card (learning steps every minute) | ≤ 2 learn + 1 relearn credited per card per day; lapses need a prior interval ≥ 2 days and are ≤ 20% of a day | none needed |
| 6 | Custom study / filtered deck cramming / Grade Now | Type 3: ignored. RESOLVED: Grade Now writes type 3 rows with an ease (finding from the adversarial review; I did not re-check the source) | none needed |
| 7 | Select 1,000 cards, "set due date: today", review them | Manual rows mark cards tampered; spacing runs from the last ease > 0 row: no credit | none needed |
| 8 | Forget/reset cards so they become new again | Tampered; type 0 rows of a tampered card get no new-card credit | none needed |
| 9 | Delete and reimport a deck (new cids) | New cards: 50/day ledger. Reimport with scheduling: non-new and unknown, so uncounted | none needed |
| 10 | Forged snapshot of 1,500 cards on 1-day intervals, one Good row per card per day, ivl 1→1, 2–8 s, ~10% Again (the reviewer's cheapest farm) | Rule 7: Good/Easy ending at ivl 1 with no growth earns nothing and flags the card; lapses on 1-day cards earn nothing | Closed |
| 11 | Same farm but intervals grow 1→2→4→8 | Each card earns less and less often; sustaining 1,500/day needs a forged pool of roughly 15,000+ cards with realistic due spread. Front-loaded snapshots are flagged | Snapshot flags added (size, front-loading). Still bounded by the ramp |
| 12 | FSRS "reschedule" with desired retention 0.99 to shrink every interval, then review the extra due cards | Type 5 may shorten `expected_ivl` only to 50% of the last scored interval | Floor added. The extra reviews are real study at higher retention, which is accepted |
| 13 | Shift `utc_offset_min` or rollover to squeeze two days' caps into ~24 h | Offset fixed at pairing, one change per 7 days, plus the 1,500-per-24h-window cap on row timestamps | none needed |
| 14 | Backfill: after 24 days, post forged rows for every past day to collect the old days' caps | Accepted if each card chain passes; the backfill flag fires; day credit is capped by the ramp that applied on that day | New hole, see below |
| 15 | Inflate the ramp by backfilling rows on many past days | `review_days` ≤ calendar days since `paired_at` (server clock) | none needed |
| 16 | 5 Anki profiles posting to one player | Shared ledger, ramp and daily cap per player | none needed |
| 17 | Edit `time_ms` to 4,000 on 0.3 s taps | Not detectable per row; the metronome flag catches only a constant fake time | Can't be fixed fully |
| 18 | Wipe the salt and re-pair to reset flags or resend a bigger forged snapshot | Card records reset, the ramp resets to 300, and `paired_at` moves forward | none needed |
| 19 | Flood the endpoint | Per token: max 1 batch every 20 s, ≤ 2,000 rows; wrong token: 10/h/IP (existing) | none needed |

New hole found on the second pass (row 14): dropping the freshness rule means a fake poster no longer has to run every day. It can fabricate a full month of past days in one sitting and claim each day's cap retroactively. Fix: credit for past days arriving in one batch is limited to the ramp that applied on each day, AND to a backfill budget. Rows for Anki days older than yesterday can credit at most 2 × ramp in total per arrival day. An honest phone user who syncs desktop weekly still gets most of the week, and fills the rest the next day. It also fires the backfill flag for Conner. It stops the instant month-long jackpot. It does not stop a patient forger who backfills a little each day; that is the fake-poster floor.

Smaller note: `due_day` in the snapshot needs the collection's creation day to become absolute (review cards store due as days since collection creation, CONFIRMED https://github.com/ankidroid/Anki-Android/wiki/Database-Structure). The add-on computes this locally from `col.sched.today` / collection creation time (UNVERIFIED API name; check in Anki 25.x).

Each row becomes an attack script in the proof. Cases 1 to 10, 13, 15 and 18 must show 0 extra credit. Case 14 must show the backfill budget holding.

## Recommendation

Build the revised Design B (review ledger), including the backfill budget from the second pass. Also drop `log_reviews` and the honour column path (migration 006), and replace the reused pairing code with the single-use code and device token.

Why: it's the only option that turns "type a number" into "forge a realistic, growing collection and run a scheduler simulator or bot", and even then the payoff is capped at what a diligent real student would earn. It keeps card content private, credits phone reviews whenever the desktop syncs, and costs about 4 days.

What it still can't stop (say this plainly to testers and to Conner):
- A player who forges a realistic collection and posts convincing fake reviews, or runs a randomized auto-clicker on a real deck.
- A player who honestly taps through a trivial deck. That costs real time, but they learn nothing.
- Someone else doing the reviews.
- Any data faked on the player's own computer that stays within normal-looking patterns. The client can't be trusted, and no Anki or AnkiWeb source exists to check against.

The defenses are caps, a ramp, checks against the schedule, and human review of flags, not proof.

## Final amendments (adversarial review round 2: SHIP with these)
1. **Backfill queue.** Rows over the per-arrival-day backfill budget are stored uncredited with `reason='budget'` and re-checked on every later arrival day, oldest first, until credited or past their own day's cap. Honest weekly phone-syncers lose nothing; they are just paid out over a few days. Ramp review-days count from row timestamps, including queued rows that later credit.
2. **Growth rule applies to Good and Easy only (ease 3, 4).** Anki can honestly keep a 1-day card at 1 day on Hard (review.rs `.max(1)`), so Hard reviews are credited without the growth check; Good/Easy always land on 2+ days under SM-2 and FSRS, so the stuck-card flag can't misfire on them.
3. Known floor (accepted): an imported big scheduled collection + a randomised auto-answer add-on passes every rule and reaches the cap after the ~24-day ramp. Only human review of flags and the cap limit this.
4. To check while building: the add-on API name for today's Anki day (`col.sched.today`) in a real Anki.
