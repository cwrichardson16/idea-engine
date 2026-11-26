# Gameplay

How the game works. It's modeled on how the base-builder strategy genre works (timers, builders, resources, raids, clans),
with original characters, art and names, and one big swap: **studying replaces waiting**.

---

## 1. How the genre works (reference)

| System | How it works in a typical base-builder | Why it works |
|---|---|---|
| Main building | One building's level gates everything else | Clear long-term goal |
| Resources | Collectors produce over time, storage caps them, raids steal them | Reasons to check in, raid and protect |
| Builders | Only a few upgrades at once; timers run hours to days | Scarcity and anticipation |
| Army | Troops are trained from a resource and need camp space | Preparing for attacks |
| Raids (async PvP) | Attack an AI-defended snapshot of another player's base; 1–3 stars by destruction; steal loot | The fun, skill-based part |
| Defense | You design your base layout; you get raided while offline; a shield follows | Ownership, base-design puzzle |
| Clans | Donate troops to clanmates, clan wars, co-op clan projects | Social glue and long-term retention |
| Money | Premium gems skip timers and buy resources and builders; a monthly pass; cosmetics | Impatience is the product |
| Rhythm | Timers bring you back several times a day | Habit |

**The genre's weak spot is waiting, and it charges you to skip the wait. We swap waiting for studying.**

---

## 2. Core principle

> **Money buys time and looks. Studying buys power.**

You can pay to build faster and look cooler, like any base-builder. You *can't* pay for an army, hero levels or war wins.
Those come only from studying. That keeps paying from undermining the study part, and keeps clan wars fair.

---

## 3. Setting

**You run a hospital.** Your staff is your army: med students, interns, nurses, surgeons, attendings.
Diseases besiege you when your cards go overdue, and rival hospitals raid you for funding.
The humor comes from med school and hospital life: paperwork walls, insurance-denial cannons, the nurses secretly running everything.

---

## 4. Resources

| Resource | Comes from | Spent on | Buy with money? | Can be stolen in raids? |
|---|---|---|---|---|
| **Funding** | Passive: clinics, billing office, grants | Buildings, defenses, walls | Yes (via gems) | Yes |
| **Brainpower** | **Only from Anki reviews** | Training staff, hero upgrades, research | **No** | **No** |
| **Gems** | Real money; small free trickle from achievements and study milestones | Speed-ups, Funding, extra builders, cosmetics | Yes | No |

Brainpower rules (anti-cheese, see DESIGN.md §3):
- Earned per **due** review, with a bonus for clearing the whole queue.
- Reviews answered too fast (under ~1.5s) earn nothing.
- New cards earn a capped amount per day.

---

## 5. Buildings

| Role | Building | Notes |
|---|---|---|
| Main building | **Hospital** | Tiers: Clinic → Community Hospital → Regional Hospital → Academic Medical Center → Level 1 Trauma Center |
| Builders | **Construction crew** | Start with 2; buy more with gems; a study streak can grant a temporary extra crew |
| Funding production | Outpatient Clinic, Billing Office, Grant Office | Passive, capped by storage |
| Funding storage | Endowment Vault | Raid target |
| Brainpower storage | Library | Can't be raided |
| Army camp | Call Room | Where your staff sleeps (barely) |
| Training | Lecture Hall | Trains staff with Brainpower |
| Research | Sim Lab | Upgrades staff types with Brainpower |
| Clan building | Consult Desk | Request reinforcements from clanmates |

**Defenses** (examples): Prior Authorization Wall (walls), Insurance Denial Cannon, Pager Tower (area damage),
Security Desk, Sanitizer Sprinklers (strong against disease waves).
**Traps:** Wet Floor Sign (slows attackers), Malpractice Lawsuit (big one-time damage).

---

## 6. Staff (troops)

| Unit | Role |
|---|---|
| Med Students | Cheap swarm, weak, very enthusiastic |
| Interns | Basic all-rounder, fast to train |
| Nurses | Healers. Secretly the strongest unit. |
| Surgeons | Heavy damage to buildings |
| Anesthesiologists | Put defenses to sleep (stun) |
| Radiologists | Long range, never leave the dark room |
| Pharmacists | Buff nearby staff |

**Heroes** (upgraded with Brainpower only): The Attending, The Chief Resident.
**Boosts** (one-shot battle effects): Coffee (speed), Code Blue (heal), and so on.

---

## 7. The Outbreak (PvE siege from your backlog)

This is the mechanic the genre doesn't have.

- Overdue cards spawn a **disease horde** that gathers at your gates. You watch it grow as cards come due during the day.
- **Every review knocks germs off the horde, live.**
- When the Anki day rolls over, whatever is left **attacks your hospital** in an automatic battle against your defenses. You can watch the replay.
- **Soft losses only:** the horde steals some Funding and slows production for a few hours ("quarantine"). It never destroys buildings or progress.
- **Clean Bill of Health:** clear your due queue and there's no outbreak, plus a bonus.
- **Comeback bonus:** clearing a big backlog pays extra.
- **Vacation shield:** pause outbreaks for exam week, holidays or sickness.

The outbreak uses the same battle engine as raids, so building it first also builds the combat system.

---

## 8. Raids (async PvP)

- Train staff with Brainpower, then attack an AI-defended snapshot of another player's hospital.
- 1–3 stars by destruction percentage; steal Funding.
- After being raided, you get a shield for a while.
- **Rank ladder:** MS1 → MS2 → MS3 → MS4 → Intern → Resident → Fellow → Attending → Chief → Dean.
- Because staff costs Brainpower, **you can't raid without studying.** Wanting to attack is the motivation to review.

---

## 9. Clans

- **Consults:** request staff at your Consult Desk; clanmates donate staff they trained by studying. Your studying helps your friends.
- **Clan wars** (two days: prep day, then battle day):
  - Each member gets 2 attacks, and stars add up.
  - **Study stars:** each member who clears their due queue on war days earns the clan +1 star.
    Wars reward both attack skill and studying. Study stars use completion, not raw counts, so big and small decks compete fairly.
- **Clan project:** a shared Teaching Hospital wing. Members contribute Brainpower, and it unlocks perks for the whole clan.
- Later: clan leagues.

---

## 10. Money

| Product | What it is |
|---|---|
| **Gem packs** | Usual tiered packs (small to large) |
| **White Coat Pass** (~$5/month) | Monthly track of study challenges, with cosmetics and small boosts (faster builds, more Funding) |
| **Cosmetics** | Hospital skins, staff skins, scrub colors, decorations |
| **Construction crew packs** | Permanent extra builders |

**Never sold:** Brainpower, staff, hero levels, war stars. **No loot boxes.**
Payments go through web checkout (e.g. Stripe) from the add-on; app-store purchases come with the phone app.
To check: AnkiWeb's rules for add-ons with paid features.

---

## 11. A med student's day

| When | What happens in the game |
|---|---|
| Morning (1 min) | See last night's outbreak result, collect Funding, start upgrades |
| Study blocks | Brainpower piles up, and the horde at the gates shrinks with every review |
| Study break (3 min) | Train staff, do a raid, donate to clanmates |
| Night | Clear the queue → Clean Bill of Health, no outbreak tonight |

---

## Open questions

- **Name.**
- Should clearing your queue also give a short **raid shield**? It's a strong study incentive, but too strong would kill PvP.
- Call clans "Programs" (as in residency programs)?
- Numbers: how many reviews to train an army, timer lengths, Funding rates. Tuned during the prototype.
