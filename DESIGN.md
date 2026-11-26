# Design framework

Product, tech and roadmap. **Game mechanics live in [GAMEPLAY.md](GAMEPLAY.md).**

Each section notes what's **decided**, what's a **proposed default** (we go with it unless someone objects), and what's still **open**.

---

## 1. Product shape

- **v1 is an Anki add-on.** The game lives inside desktop Anki and reacts to reviews as they happen.
- **Multiplayer from the start:** accounts and a server exist early, so raids and clans aren't bolted on later.
- **Later:** a phone app reusing the same game code. That's also where app-store purchases come in.

Decided.

---

## 2. Core loop

Modeled on how base-builders work, with studying replacing waiting. Details are in GAMEPLAY.md.

```
study in Anki ──► Knowledge (skips timers) + daily study bonus (resources, ores) + clan study stars
passive gold/elixir + raid loot ──► build, train, research ──► raids / clan wars
gems (paid) skip timers too

overdue cards ──► siege (enemy army attacks your base)
```

Decided:
- Medieval fantasy setting, with light med-school humor in names and flavor.
- Money works like CoC (gems, offers, pass, cosmetics). Study-only things are never for sale: Knowledge, study stars, clan study challenge progress.

Proposed:
- **Soft sieges:** they never destroy buildings or progress. Plus a comeback bonus and a vacation shield.

---

## 3. Study → reward rules (anti-cheese)

- Reward reviews of **due** cards, so the scheduler caps daily earnings.
- **No accuracy rewards** (grading is self-reported, so rewarding "Good" teaches lying).
- Reviews answered too fast (e.g. under ~1.5s) earn nothing.
- Rewards from new cards are capped.
- Rewards come from Anki's own review log, which is harder to fake than taps in our UI.

A determined person can still cheat a local app. Keep real-world stakes low (no cash prizes).

---

## 4. Tech architecture (proposed defaults)

| Piece | Choice | Why |
|---|---|---|
| Add-on | Python, Anki add-on API (`gui_hooks.reviewer_did_answer_card`, review log) | Standard way to hook Anki |
| Game UI | TypeScript + **Phaser** (2D game framework) in Anki's built-in web view | Web tech, reusable in the phone app |
| Server | **Supabase** (Postgres, auth, realtime, edge functions) | Cheap, fast to build on, realtime for clans |
| Game state | **Server-authoritative** (resources, timers, purchases) | Stops save-file editing |
| Battles | **Deterministic simulation**, so the server can re-run a raid and confirm the result | Stops faked raid results |
| Offline | Queue review events locally and sync when online | Anki is often used offline |
| Login | Email code typed into the add-on | No browser redirect dance inside Anki |
| Payments | Merchant of record (Paddle or Lemon Squeezy) web checkout, with gems granted server-side | Handles sales tax/VAT; add-ons can't use app stores |

Anki is AGPL-licensed, so the add-on will most likely be open source. The server stays private, and it holds everything that matters (economy, purchases).

---

## 5. Art

Open. Simple shapes for the prototype. Then asset packs, AI-generated art or a commissioned artist.
All art is original. No look-alikes of existing games.

---

## 6. Roadmap

| Phase | Goal | Contents |
|---|---|---|
| **0. Prototype** | Does the loop feel good? | Add-on hooks reviews, earns Knowledge, one building on a timer, local only |
| **1–4. Game phases** | The game | See GAMEPLAY.md §10: core base + raids → clans → heroes/equipment → leagues/capital |
| **5. Phone app** | Play away from desktop | Same game code, app-store purchases |

---

## 7. Decision log

| # | Decision | Status |
|---|---|---|
| 1 | First audience: med students | Decided |
| 2 | v1 as an Anki add-on | Decided |
| 3 | Clans and multiplayer in scope | Decided |
| 4 | Modeled on base-builder genre mechanics, with original characters/art/name | Decided |
| 5 | Setting: medieval fantasy, light med humor | Decided |
| 6 | Money like CoC; study-only things never sold | Decided |
| 7 | Mechanic decisions M1–M8 | See GAMEPLAY.md §11 |
| 8 | Build order: see GAMEPLAY.md §10 | Proposed |
| 9 | Tech stack (section 4) | Proposed |
| 10 | Name | **Open** |
| 11 | Art sourcing | Prototype with one free pack; one consistent (likely commissioned) style before launch. See OPEN_SOURCE.md |
| 12 | Gems + payments from day one (server + accounts in v1; merchant-of-record checkout) | Decided |
