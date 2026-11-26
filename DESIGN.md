# Design framework

Working doc. Each section ends with what's **decided**, what's a **proposed default** (we go with it unless someone objects),
and what's still **open**.

---

## 1. Product shape

- **v1 is an Anki add-on.** The game lives inside desktop Anki and reacts to reviews as they happen.
- **Multiplayer from the start:** accounts and a server exist from early on, so clans aren't bolted on later.
- **Later:** a phone/web companion app reusing the same game code (check your base, collect, clan stuff).

Decided: add-on first, clans and multiplayer are in scope.

---

## 2. Core loop

```
study in Anki ──► earn study currency ──► speed up builds / train defenders
      ▲                                              │
      │                                              ▼
 backlog grows ◄── skip days        base grows ──► join clan efforts / wars
      │
      ▼
   SIEGE (overdue cards attack your base)
```

- **Passive currency** trickles in over time. It builds basic stuff, so you *can* progress without studying, just slowly.
- **Study currency** comes only from reviews. It speeds things up, unlocks top-tier upgrades and trains defenders.
- **The siege:** overdue reviews become an attacking force. Clearing them fights it off.
  Not studying is self-correcting inside the game; no extra punishment needed.

Proposed defaults:
- Two currencies, one passive and one from studying. More currencies can come later if the economy needs them.
- **Soft sieges:** a siege can steal resources and slow production, but it never destroys buildings or wipes progress.
  Med students already have guilt spirals; the game must not add to them.
- **Comeback bonus:** clearing a big backlog pays extra. Recovering should feel great.
- **Vacation shield:** pause sieges for a set number of days (exam week, holidays, being sick).

---

## 3. Study → reward rules (anti-cheese)

- Reward **reviews of due cards**, so the scheduler caps daily earnings.
- **No accuracy rewards** (grading is self-reported, so rewarding "Good" teaches lying).
- **Ignore button-mashing:** reviews answered too fast (e.g. under ~1.5s) earn nothing.
- **Cap the reward from new cards**, so nobody adds junk cards to farm.
- Rewards come from Anki's own review log, which is harder to fake than taps in our UI.

Accept that a determined person can cheat a local app. Keep the stakes low enough (no cash prizes) that it doesn't matter.

---

## 4. Multiplayer

| Feature | What it is | Proposed phase |
|---|---|---|
| **Clans** | Groups of ~5–30 (a class, a study group) | Phase 2 |
| **Clan project** | Co-op: members' reviews fuel a shared clan building | Phase 2 |
| **Clan wars** | Weekly clan vs clan, scored on **% of due reviews cleared** | Phase 2 |
| **PvP raids** | Attack another player's base asynchronously, with study-trained troops | Phase 3 |
| **Clan chat** | Skip at first: classes already have GroupMe/Discord, and chat needs moderation | Later / maybe never |

**Fairness:** score clans on *completion %*, not raw review count. An M1 with 200 due cards and an M3 with 900 can then compete fairly.

Proposed: co-op and clan wars before PvP. Cooperation fits a cohort that studies together; raids are the hardest part to balance.

---

## 5. Tech architecture (proposed defaults)

| Piece | Choice | Why |
|---|---|---|
| Add-on | Python, Anki add-on API (`gui_hooks.reviewer_did_answer_card`, review log) | Standard way to hook Anki |
| Game UI | TypeScript + **Phaser** (2D game framework), rendered in Anki's built-in web view | Web tech, reusable for the phone/web app later |
| Server | **Supabase** (Postgres, auth, realtime, edge functions) | Cheap, fast to build on, has realtime for clans |
| Game state | **Server-authoritative** for anything multiplayer touches | Stops people editing local save files to win clan wars |
| Offline | Queue review events locally and sync when online | Anki is often used offline |
| Login | Email code typed into the add-on | No browser redirect dance inside Anki |

Anki is AGPL-licensed, so the add-on will most likely be open source. The server can stay private.

---

## 6. Theme & art

Open. Options:

- **A. Immune system:** your base is a body, and the backlog is an infection.
  Very on-brand for med students and makes the siege intuitive. Narrower when expanding beyond med.
- **B. Fantasy/kingdom:** familiar and general. Easy to expand to other audiences, but less distinctive.
- **C. Hospital tycoon:** build a hospital, and the backlog is a patient surge. Tycoon-flavoured; the siege metaphor is weaker.

Art sourcing, also open: simple shapes for the prototype, then asset packs, AI-generated art or a commissioned artist.
All art must be original. No look-alikes of existing games.

---

## 7. Money

Open, not urgent. One principle proposed now: **never sell what studying earns.**
If gems can be bought, studying stops mattering. Possible later options: cosmetics, clan perks, donations/Patreon (common for Anki add-ons).

---

## 8. Roadmap

| Phase | Goal | Contents |
|---|---|---|
| **0. Prototype** | Does it feel good? | Add-on hooks reviews, earns currency, one building on a timer, local only |
| **1. Single-player** | A real game loop | Accounts + server, several buildings, both currencies, siege, vacation shield |
| **2. Clans** | Social hook | Clans, clan project, weekly clan wars |
| **3. PvP** | Competition | Async raids |
| **4. Companion app** | Play away from desktop | Phone/web app reusing the game code |

---

## 9. Decision log

| # | Decision | Status |
|---|---|---|
| 1 | First audience: med students | Decided |
| 2 | v1 as an Anki add-on | Decided |
| 3 | Clans and multiplayer in scope | Decided |
| 4 | Theme | **Open** |
| 5 | Multiplayer order: co-op/clan wars before PvP | Proposed |
| 6 | Soft sieges (never destroy progress) | Proposed |
| 7 | Progress possible without studying, just slow | Proposed |
| 8 | Tech stack (section 5) | Proposed |
| 9 | Art sourcing | **Open** |
| 10 | Monetization ("never sell study currency") | Proposed |
| 11 | Name | **Open** |
