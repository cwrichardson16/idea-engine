# Untitled study base-builder (name TBD)

A medieval base-builder strategy game for med students, played inside Anki.
**The pitch: a base-builder where studying is the power source.** Your reviews speed up your builds and power your daily rewards; your overdue cards attack your base.

> Status: early concept. Nothing is built yet. v1 will be an Anki add-on, with clans and multiplayer planned.

## Core loop

- You build and upgrade a base. Like most base-builders, upgrades take real time.
- Studying is the accelerator: finishing your due reviews speeds up timers, completes builds and refills resources.
- You *can* play without studying, but progress is slow. Studying is what makes it fast.

## Anti-cheese principles

Self-graded flashcards are easy to fake, so rewards must not invite lying:

- **Reward clearing the due queue, not raw card count.** The spaced-repetition scheduler caps how much you can do per day, so you can't farm it.
- **Don't reward accuracy on self-graded cards.** That only teaches people to press "Good".
- **Ignore button-mashing:** reviews answered too fast earn nothing.
- **Cap rewards from new cards**, so nobody adds thousands of junk cards to farm.

## Studying happens in Anki

- Students keep studying in Anki exactly as they do now. The game is an **Anki add-on** that reacts to every review.
- Reviews done on phones (AnkiMobile/AnkiDroid) sync to desktop Anki and get counted when desktop Anki syncs.
- Later options for non-Anki users: our own study mode (FSRS scheduler), `.apkg` / CSV / Quizlet imports, AI-generated cards.

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

## Design

- [CLASH_REFERENCE.md](CLASH_REFERENCE.md): how Clash of Clans works, system by system (the model for our mechanics).
- [V1.md](V1.md): the simple first version (gold, elixir, buildings, battles, studying) and its decisions.
- [OPEN_SOURCE.md](OPEN_SOURCE.md): engines, art, and Anki add-ons we can use or learn from.
- [GAMEPLAY.md](GAMEPLAY.md): our version of each system, where studying plugs in, and open mechanic decisions.
- [DESIGN.md](DESIGN.md): product, tech, roadmap and decision log.
