# Gameplay

Our version of each Clash of Clans system (see [CLASH_REFERENCE.md](CLASH_REFERENCE.md) for how CoC works).

- Mechanics first. **All names are placeholders**; theme and names come last.
- Medieval fantasy setting. Our own characters, art and name.
- Where it says **study hook**, that's where studying plugs into the system.

---

## 1. Where studying plugs in (summary)

| CoC system | Our version | Study hook |
|---|---|---|
| Gems skip timers | Same, plus **Knowledge** (earned only by studying) also skips timers | Studying = free speed-ups |
| Daily star bonus | **Daily study bonus**: clear your due cards → resources + ores | Daily reason to study |
| Ores → hero equipment | Same, but ores come mostly from study bonuses, streaks and wars | Hero strength tracks consistent studying |
| Magic items | Same, with study milestones as a main source | Big study days pay out |
| Getting raided while offline | Same, **plus the backlog siege** (overdue cards attack you) | Not studying has an in-game cost |
| Clan wars | Same, **plus study stars** | Studying helps your clan win |
| Clan Games | **Clan study challenges** (collective review goals) | Clan studies together |
| Clan Capital | Capital funded by members' Knowledge | Clan builds with studying |

Everything else (building, raiding, defending, trophies, donations) works like CoC.

---

## 2. Resources

| Resource | How you get it | What it's for | Buy with gems? | Stolen in raids? |
|---|---|---|---|---|
| **Gold** | Mines, raid loot | Buildings, defenses, walls | Yes | Yes |
| **Elixir** | Collectors, raid loot | Troops, spells, research, army buildings | Yes | Yes |
| **Dark elixir** (unlocks mid-game) | Drills, raid loot | Strong troops, hero upgrades | Yes | Yes |
| **Gems** | Real money; small free amounts | Skip timers, buy resources, builders, cosmetics | n/a | No |
| **Knowledge** | **Only from Anki reviews** | Skip timers (like gems), clan capital contributions | **No** | **No** |
| **Ores** (common / rare / epic) | Daily study bonus, streaks, wars, events | Hero equipment | Limited (offers) | No |
| **Magic items** | Study milestones, clan challenges, the pass, events | Instant finishes, boosts | Some | No |

**Knowledge rules** (anti-cheese, see DESIGN.md §3): earned per **due** review, bonus for clearing the queue,
nothing for reviews faster than ~1.5s, new cards capped.

---

## 3. Base, builders & progression

Same as CoC:
- Town Hall level caps everything; upgrading it unlocks new stuff.
- Resource buildings, army buildings, lab, hero buildings, clan castle, defenses, walls, traps.
- Builders are the bottleneck; the lab does one research at a time.
- Timers go from minutes early on to days late game.

**Study hook:** Knowledge skips timers. Paying players use gems; studying players use Knowledge; most will use both.

---

## 4. Attacking

Same as CoC: deploy troops with targeting preferences, spells, heroes, 3-minute battles, 3 stars, loot, trophies.

**Daily study bonus** replaces the daily star bonus: clear your due cards for the day → resources, ores, sometimes a magic item.

Optional idea to test later: **flashcard spells** (answer a card mid-battle to power up a spell). It might be fun, or it might be annoying mid-raid.

---

## 5. Defending

Same as CoC: base layout, offline raids, replays, revenge, shields, clan castle defense, separate war base.

**Study hook: the backlog siege.**
- Overdue cards become an enemy force gathering outside your walls. It grows as cards come due and shrinks with every review.
- At Anki's day rollover, whatever's left attacks your base automatically, using the same battle engine as raids.
- **Soft losses:** it steals some resources. It never destroys buildings.
- Clear your queue and there's no siege. Vacation shield for exams and sickness. Comeback bonus for clearing a big backlog.

---

## 6. Leagues

Same as CoC: trophy leagues, loot bonus by league, a top league with its own daily format, season resets.
Single-player campaign as the tutorial.

---

## 7. Clans

| CoC | Ours |
|---|---|
| Clan, roles, levels and perks | Same |
| Chat | Skip at first (classes already have GroupMe/Discord; chat needs moderation) |
| Donations | Same |
| Clan Wars (prep day, battle day, 2 attacks each) | Same **+ study stars**: each member who clears their due cards on war days adds +1 star (completion-based, so big and small decks are fair) |
| Clan War Leagues | Same (later) |
| Clan Games | **Clan study challenges**: monthly collective goals ("clan clears 20,000 reviews", "everyone keeps a 5-day streak"), unlocking reward tiers |
| Clan Capital + Raid Weekends | Capital funded by members' Knowledge; raids later |

---

## 8. Money

Same as CoC:
- Gem packs.
- Special offers.
- Monthly pass (challenges include study goals).
- Cosmetics: hero skins, sceneries, decorations.
- Extra builders.

**Study-only, never for sale:** Knowledge, study stars, clan study challenge progress. No loot boxes.

---

## 9. What we're not doing (for now)

- **Builder Base** (second village): cut. Too much extra scope.
- **Clan chat**: cut at first.
- **Clan Capital Raid Weekends**: later.

---

## 10. Phases

CoC has a decade of content. We build a slice, then add systems in order. **v1 is detailed in [V1.md](V1.md).**

| Phase | Systems |
|---|---|
| **v1** | Town Hall, gold + elixir, ~10 building types, 3 troops, battles vs computer bases, **Knowledge skips**, **daily study bonus**, backlog siege |
| **v2** | Accounts + server, PvP raids, stars, trophies, shields, walls, gems |
| **v2.5** | Clans: donations, clan wars with study stars, clan study challenges |
| **v3** | Dark elixir, heroes, ores + equipment, magic items, monthly pass |
| **v4** | Clan War Leagues, Clan Capital, events, top league |

---

## 11. Mechanic decisions

| # | Question | Recommendation | Status |
|---|---|---|---|
| M1 | What does Knowledge do? | Skip timers like gems, and fund the clan capital | Open |
| M2 | Daily study bonus instead of daily star bonus? | Yes | Open |
| M3 | Backlog siege, and how harsh? | Yes, soft: steals resources, never destroys | Open |
| M4 | Study stars in clan wars? | Yes, on top of attack stars | Open |
| M5 | Clan Games become clan study challenges? | Yes | Open |
| M6 | Cut Builder Base? | Yes | Open |
| M7 | Flashcard spells mid-battle? | Prototype later, maybe | Open |
| M8 | How much is one review worth? | Tune during the prototype | Later |
