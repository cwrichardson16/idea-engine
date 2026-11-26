# Study Clash (working title)

A spaced-repetition study app fused with a base-building tycoon game.
**The pitch: a base-builder where the gems are flashcards.**

> Status: early concept. Nothing is built yet; this file tracks the design as it takes shape.

## Core loop

- You build and upgrade a base. Like most base-builders, upgrades take real time.
- Studying is the accelerator: finishing your due reviews speeds up timers, completes builds and refills resources.
- You *can* play without studying, but progress is slow. Studying is what makes it fast.

## Anti-cheese principles

Self-graded flashcards are easy to fake, so rewards must not invite lying:

- **Reward clearing the due queue, not raw card count.** The spaced-repetition scheduler caps how much you can do per day, so you can't farm it.
- **Don't reward accuracy on self-graded cards.** That only teaches people to press "Good".
- **Bonus rewards for checkable formats** (typed answers, multiple choice).
- **Cap rewards from new cards**, so nobody adds thousands of junk cards to farm.

## Study engine & imports

- Studying happens in-app, scheduled with **FSRS** (the open-source algorithm modern Anki uses).
- Importing is how people get their existing material in:

| Source | Feasibility | Phase |
|---|---|---|
| Anki `.apkg` file (decks + review history) | Easy: zipped SQLite | v1 |
| CSV / Quizlet copy-paste | Easy | v1 |
| AI-generated cards from notes/PDFs | Easy | v1 / v2 |
| Anki add-on that reports reviews to our server | Doable | later |
| AnkiDroid API | Android only | later |
| AnkiConnect | Desktop only, Anki must be open | maybe |
| AnkiWeb / AnkiMobile (iOS) | No public API | no |

## First audience: med students

Chosen as the starting audience (expand later to nursing, pharmacy, law, language learners).

- **Huge daily load:** 300–800+ Anki reviews a day, burnout, and missing a day snowballs into a backlog.
- **Already locked into Anki** (AnKing deck, add-ons, FSRS), so asking them to switch apps is a hard sell.
  → Likely means the game should be a **companion that tracks their Anki reviews**, not a replacement study app.
- **Time-poor:** the game should be a few minutes of check-ins as a reward, never a time sink.
- **Built-in social groups:** classmates share the same deck and the same exams (Step 1/2), so clans map naturally to classes and study groups.

### Mechanic idea: the siege

Overdue reviews are an army at your gates. The bigger your backlog, the bigger the siege; clearing reviews fights it off.
That turns the most painful part of Anki (the backlog) into the game's core tension.

## Open questions

1. **Companion to Anki or replacement?** Leaning companion for med students (see above).
2. **PvP raids, or building only?** Middle ground: clans where members' studying adds to a shared project.
3. **Can you progress with zero studying?** Slowly, or not at all?
4. **Platform:** phone app or web first? Web is faster to prototype; studying mostly happens on phones.
