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
study in Anki ──► Brainpower ──► train staff ──► raid other hospitals / clan wars
                                                     │
passive Funding ──► build & upgrade hospital ◄───────┘ (stolen Funding)
      ▲
gems (paid) speed it up

overdue cards ──► Outbreak (disease horde attacks your hospital)
```

Decided:
- Hospital setting with med-school humor.
- **Money buys time and looks; studying buys power.** Gems are for sale. Brainpower, staff, heroes and war stars never are.

Proposed:
- **Soft outbreaks:** they never destroy buildings or progress. Plus a comeback bonus and a vacation shield.

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
| Payments | Web checkout (e.g. Stripe), with gems granted server-side | Add-ons can't use app stores |

Anki is AGPL-licensed, so the add-on will most likely be open source. The server stays private, and it holds everything that matters (economy, purchases).

---

## 5. Art

Open. Simple shapes for the prototype. Then asset packs, AI-generated art or a commissioned artist.
All art is original. No look-alikes of existing games.

---

## 6. Roadmap

| Phase | Goal | Contents |
|---|---|---|
| **0. Prototype** | Does the loop feel good? | Add-on hooks reviews, earns Brainpower, one building on a timer, local only |
| **1. Single-player** | A real game | Accounts + server, hospital and buildings, Funding/Brainpower/gems, Outbreak (builds the battle engine), gem purchases |
| **2. Raids** | Competition | Train staff, async PvP raids, rank ladder, shields |
| **3. Clans** | Social hook | Consults, clan wars with study stars, clan project, White Coat Pass |
| **4. Phone app** | Play away from desktop | Same game code, app-store purchases |

---

## 7. Decision log

| # | Decision | Status |
|---|---|---|
| 1 | First audience: med students | Decided |
| 2 | v1 as an Anki add-on | Decided |
| 3 | Clans and multiplayer in scope | Decided |
| 4 | Modeled on base-builder genre mechanics, with original characters/art/name | Decided |
| 5 | Setting: hospital, med-school humor | Decided |
| 6 | Gems for sale; Brainpower, staff, heroes and war stars never sold | Decided |
| 7 | Soft outbreaks (never destroy progress) | Proposed |
| 8 | Build order: Outbreak → raids → clans | Proposed |
| 9 | Tech stack (section 4) | Proposed |
| 10 | Name | **Open** |
| 11 | Art sourcing | **Open** |
| 12 | Raid shield for clearing your queue? | **Open** |
