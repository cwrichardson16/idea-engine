# How Clash of Clans works (reference)

A breakdown of the systems in Clash of Clans (CoC), the game we're modeling mechanics on.
We copy *how the systems work*, not CoC's names, characters, art or look.
Current as of late 2026 (Town Hall 19, six heroes). Small details like exact numbers may be slightly off.

---

## 1. The three "worlds"

| World | What it is |
|---|---|
| **Home Village** | The main game: build, train, raid. Almost everything below is about this. |
| **Builder Base** | A separate second village with its own resources and its own battle mode. |
| **Clan Capital** | A shared village the whole clan builds together, with weekend raids against other clans. |

---

## 2. Resources

| Resource | How you get it | What it's for |
|---|---|---|
| **Gold** | Gold mines (passive), raid loot | Most buildings, defenses, walls |
| **Elixir** | Elixir collectors (passive), raid loot | Normal troops and spells, research, army buildings, walls (later) |
| **Dark Elixir** | Drills (passive, unlocked mid-game), raid loot | Strong "dark" troops and spells, hero upgrades |
| **Gems** | Real money; small free amounts from obstacles, achievements, events, the pass | Skip any timer, buy resources, buy builders, cosmetics |
| **Ores** (3 kinds: common, rare, epic) | Daily star bonus, clan wars, events, trader | Upgrading hero equipment |
| **Magic items** | Clan Games, events, league medals, raid medals, the pass | Instant finishes (books, hammers), timed boosts (potions), storage fills (runes), etc. |
| **League medals** | Clan War Leagues | Shop for magic items and ores |
| **Capital Gold / Raid Medals** | Clan Capital raids and contributions | Upgrading the Clan Capital / personal rewards shop |
| **Builder Gold / Builder Elixir** | Builder Base only | Builder Base only |

**The key pattern:** cheap common resources (gold/elixir) → a rarer mid-game resource (dark elixir) →
special currencies for special systems (ores for equipment, medals for shops). Each new system gets its own currency,
so there's always something to work toward.

---

## 3. Buildings

| Type | Buildings | Notes |
|---|---|---|
| **Town Hall** | 1 | Its level caps the max level of everything else. Upgrading it unlocks new buildings and levels. |
| **Resource** | Mines, collectors, drills, storages, Treasury | Storages cap how much you can hold; storages and collectors are what raiders steal from |
| **Army** | Army camps (capacity), barracks, dark barracks, spell factories, siege workshop | Train troops, spells and siege machines |
| **Research** | Laboratory | Upgrades troop and spell levels; runs one research at a time |
| **Heroes** | Hero Hall, Pet House, Blacksmith | Heroes are permanent units that level up; pets attach to heroes; the Blacksmith upgrades hero equipment |
| **Clan** | Clan Castle | Holds donated troops for attack *and* defense; stores clan loot |
| **Defenses** | Lots: single-target, splash, air, anti-air, long-range, hidden, high-damage, etc. | Different defenses counter different troop types |
| **Walls** | Walls | Cheap individually, but hundreds of them; slow attackers and channel their path |
| **Traps** | Bombs, spring traps, air mines, etc. | Hidden until triggered |
| **Builders** | 5–6 builder huts + helpers | Each builder does one upgrade at a time. **Builders are the main bottleneck of the whole game.** |

---

## 4. Progression & timers

- Everything takes time: early upgrades take seconds or minutes, late ones take **days to weeks**.
- **Bottlenecks:** builders (how many upgrades at once), the lab (one research at a time), resources (enough to pay), Town Hall level (the caps).
- **"Rushing"** means upgrading Town Hall before maxing everything else. It's tempting, but your base ends up weak for its level.
- **Skips:** gems skip anything; magic items skip or shorten specific things; temporary boosts speed up builders or research.
- Fully maxing a Town Hall level takes **months**. There are 19 Town Hall levels. That's the long-term spine.

---

## 5. Attacking (async PvP)

- You attack a **snapshot** of another player's base. They aren't online; their defenses fight automatically.
- **Deploy** troops around the edge. Troops act on their own with **targeting preferences**:
  some go for the nearest building, some only target defenses, some go for resources, some break walls.
- **Spells** (heal, rage, freeze, jump, etc.) are cast on the battlefield.
- **Heroes** have active abilities and equipment. **Siege machines** carry clan troops deep into the base.
- **3 minutes** per attack.
- **Stars:** 1 for 50% destruction, 1 for destroying the Town Hall, 1 for 100%.
- **Loot:** you steal a percentage of the defender's stored and uncollected resources.
- **Trophies:** win and gain, lose and drop. Trophies set your league and matchmaking.
- **Daily star bonus:** earn a few stars a day for bonus resources and ores. It's a daily login reason.

---

## 6. Defending

- You **design your base layout**: where defenses, walls, storages and the Town Hall go. Base design is a puzzle and a big part of the fun.
- You get attacked while offline. Afterwards you can watch the **replay** and get a **revenge** option.
- **Shield:** after a heavy defeat you get a shield (roughly 12–16 hours depending on damage) so you can't be attacked again right away.
- The **Clan Castle** also defends with donated troops.
- Separate layouts for war bases.

---

## 7. Matchmaking & leagues

- **Trophy leagues:** a ladder of leagues from low to top. Higher leagues give bigger loot bonuses.
- The **top league** works differently: a fixed number of attacks and defenses per day, and season-end rankings.
- Single-player **campaign** maps against AI villages (tutorial and extra loot).

---

## 8. Clans

- Up to **50 members**, with roles (leader, co-leaders, elders, members) and **clan chat**.
- **Donations:** request troops, spells and siege machines; clanmates fill your Clan Castle.
- **Clan level & perks:** earned through clan activity.
- **Clan Wars:** a preparation day, then a battle day. Each member gets **2 attacks** against the other clan's war bases. Most total stars wins.
- **Clan War Leagues:** monthly. 8 clans in a group, 7 rounds, 1 attack per member per day. Rewards league medals.
- **Clan Games:** monthly event. Members complete challenges that add to a shared clan score, which unlocks reward tiers (magic items).
- **Clan Capital:** a shared village. Members contribute Capital Gold to upgrade it. On **Raid Weekends**, members attack other clans' capitals together.
- **Friendly challenges / friendly wars:** practice battles with no stakes.

---

## 9. Money

| Product | What it is |
|---|---|
| **Gem packs** | Tiered from small to large |
| **Special offers** | Time-limited bundles (resources, magic items, gems), often targeted at your progress |
| **Gold Pass** (monthly) | Reward track from monthly challenges + perks (faster builder/research boosts, extra star-bonus rewards) + an exclusive hero skin |
| **Event passes** | Same idea, tied to events |
| **Cosmetics** | Hero skins, base sceneries (whole-map themes), decorations |
| **Builders** | Extra builder huts can be bought with gems |

**What players pay for:** impatience (skips), convenience (passes and offers), and looks (skins, sceneries).

---

## 10. Retention loops

| Cadence | What brings you back |
|---|---|
| **Several times a day** | Upgrades finishing, collectors filling, army ready |
| **Daily** | Star bonus, top-league attacks |
| **Weekly** | Clan wars, Raid Weekend |
| **Monthly** | Gold Pass season, Clan Games, Clan War Leagues, league season end |
| **Every few weeks** | Events with special troops and event passes |
| **Months / years** | Town Hall progression, maxing |

---

## 11. Why it works

- **Always something in progress.** Builders and the lab are almost never idle, so there's always a reason to check in.
- **Short sessions.** A check-in is 1–2 minutes; an attack is 3 minutes.
- **Two kinds of fun:** building (planning, collecting) and attacking (skill, strategy).
- **Social obligation.** Your clan needs your war attacks and donations.
- **Layered goals.** Daily, weekly, monthly and long-term goals all overlap.
- **Deep progression.** Years of content.
