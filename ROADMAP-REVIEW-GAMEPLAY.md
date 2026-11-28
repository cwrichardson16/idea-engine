# Roadmap review: gameplay (skeptical Clash producer view)

Reviewed: ROADMAP-DRAFT.md against the goal "about 10 study-heavy students play a robust Clash-like for 7+ days, real Anki reviews boost them, $0".
Method: read STATE.md, SPEC-2.md, SPEC-3.md, and the code (village/logic/src/economy.rs, battle.rs, fight.rs, lib.rs, village/app.contract, village/supabase/). Tags: CONFIRMED (file:line) means I read it in code; UNVERIFIED means I did not run or see it.

## What exists today (checked in code)

- Buildings: 10 kinds. Keep, Mint, Farm, Treasury, Granary, Mason's Lodge, Barracks, Muster Ground, Ballista, Watchtower. CONFIRMED economy.rs:73-86, 94-105.
- Levels 1-8, the Keep gates every other building's level. CONFIRMED economy.rs:11, 717-719.
- Builders: 1 + one Mason's Lodge, max 1 lodge, so 2 builders forever. CONFIRMED economy.rs:595-600, 218.
- Troops: 3 (Spearman, Bowman, Lancer). No troop levels, no lab. CONFIRMED economy.rs:245-252, battle.rs:113-121 (stats take no level).
- Defenses: 2 (Ballista single target, Watchtower single target). Both scale damage by level. CONFIRMED battle.rs:102-110.
- Building HP does NOT scale with level. A level-8 Keep has the same 400 HP as level 1. CONFIRMED battle.rs:91-99.
- Troop AI: straight line to the nearest building; Lancer goes for defenses first. No walls, no pathing. CONFIRMED battle.rs:288-302, 316-337.
- Raid targets: 4 fixed camps, highest is Keep 3. Loot only the first time; the looted mask is never reset, so camps never refill (SPEC-3 T7 says they refill daily; not built). CONFIRMED battle.rs:32-88, economy.rs:388-389, 435-444; no reset anywhere (grep "looted").
- Total loot in all 4 camps: about 6,700 of each resource, once. CONFIRMED battle.rs:32-88. Gold needed to max everything: about 3.4 million. (computed from economy.rs:68-70, 154-167)
- PvP: none. No raids table, no raid RPC, no shields, trophies, defense log or revenge in code or migrations. CONFIRMED (grep across logic/src, app.contract, village/supabase/migrations 001-005).
- plans_from_town (the future PvP snapshot) gives 20% of storage plus half of mine contents, while SPEC-3 T7 says 10%. CONFIRMED battle.rs:412-431 vs SPEC-3 T7.
- Mines fill in 50 to 300 seconds and then stop (Mint L1: 2/s, holds 100; L8: 20/s, holds 6,000). CONFIRMED economy.rs:189-190, 349-360.
- Training queue: 10 troops max, one Barracks max. A level-8 army is 4 camps x 120 = 480 housing. CONFIRMED economy.rs:242, 475, 218-219, 230-233.
- Battle visuals: buildings turn to rubble; troops are capsules. No HP bars, no projectiles or shot effects, no sound. CONFIRMED lib.rs:984-997, fight.rs:157-180; grep for hp/sound in lib.rs found nothing.
- Moving buildings: not possible (no move command). CONFIRMED lib.rs command list (~498-640); SPEC-2 "Moving buildings is out of scope".
- Builder time to max everything: about 6.4 builder-days with 2 builders, and the Keep 3 tier is done in about 30 minutes. (computed from economy.rs:170-180)
- Study loop: Knowledge, chests, streaks, power hours. CONFIRMED economy.rs:1034-1097. Cloud save, restore and bug report: built and Chrome-verified per STATE.md (UNVERIFIED by me).

## 1. What a Clash player will feel is missing or broken, ranked by impact

### First 30 minutes
1. Nothing to attack after about 20 minutes. The 4 camps are cleared quickly, give loot once, and never refill. A Clash player's loop is "train, raid, upgrade"; here the raid half dies in the first session. Highest impact. CONFIRMED (above).
2. Battles are unreadable. Defenses fire invisibly: troops just lose HP and vanish, buildings pop to rubble. No HP bars, no arrows or bolts, no hit flash. Clash's whole feel lives in watching the fight. CONFIRMED lib.rs, fight.rs.
3. Collecting is a clicker, not Clash. Mines fill in under 5 minutes then stop, so you sit and tap Collect, and get almost nothing while away. Clash collectors fill over hours. CONFIRMED economy.rs:189-190.
4. Army refill is a chore: queue of 10, one Barracks. At Keep 4+ you come back to the queue again and again. CONFIRMED economy.rs:242, 218.
5. Can't rearrange the base. Clash players move buildings within minutes. Matters more once others raid you. CONFIRMED.
6. No walls. Noticed immediately ("where are the walls?"), but it does not stop play.

### Days 2-7
1. No reason to log in except study. Offline income is about one mine-fill per visit (Keep 4: about 2,400 gold per return vs a 20,000 Keep 5). Without PvP or refilling camps, days 2-7 are "tap collect for 10 minutes". CONFIRMED (computed).
2. Defense is meaningless and offense trivial at high levels. Building HP is flat and troops never level, but housing grows to 480. A Keep 8 village is about 38 buildings with roughly 10,000 total HP; 480 Spearmen deal about 4,300 damage per second. Expect every PvP raid to be an easy 3 stars, so building defenses feels pointless. Every defense is single-target, so nothing answers spam. CONFIRMED stats; outcome UNVERIFIED (not simulated).
3. No PvP yet, so no raided-while-away tension, no defense log, no revenge, no rivalry. That is the social glue for 10 friends. CONFIRMED absent.
4. Progression length is borderline. 6.4 builder-days to max, with Knowledge skips and power hours shortening it, so a heavy studier who taps actively can near max inside the test. Gold is the real gate. Probably fine for 7 days, and risky for 14. UNVERIFIED (no economy playthrough sim exists; the time-to-max test counts builder seconds only, tests/battle.rs:178-199).
5. Troop and defense variety: 3 troops and 2 defenses feels thin by day 3, but less so than items 1-3.
6. Trophies/leagues, obstacles, a 3rd builder: low impact for 10 people.

## 2. Cut or shrink for a 10-person test

- Leagues (Yeoman to Sovereign) and trophy matchmaking. With 10 players matchmaking has nothing to choose from. Keep a plain trophy number and a top-10 list (cheap, and it is the social hook).
- Obstacles to clear. Polish, no loop value.
- A 3rd builder earned by streak. Adds a balance variable mid-test; skip.
- Replays. Store the drop log (already planned) and show a text defense log; skip the visual replay.
- "Computer bases kept only for tutorial + backup". Reverse this. With 10 players, computer camps are the main raid supply (see risk 1).
- Ram. Only needed if walls ship. Trebuchet (splash) should stay, because it is the only answer to troop spam.
- In the art pass: 3 visual tiers per building and skeletal troop animation. One good model per building plus a level badge is enough for a test. (Art details belong to the art reviewer.)
- One line on Anki, not re-arguing: the owner chose the stronger anti-cheat, so keep it. For 10 hand-picked testers it is the most expensive item per tester, so time-box it at 4 days.

## 3. Is the order right?

Mostly not. Anki first is right (it is the hook and recruitment depends on it). But:
- An economy and combat fix pass must come before PvP. PvP built on flat building HP, 20% loot, 5-minute mines and a 10-troop queue will just ship the imbalance to real people. These are small constant and rule changes, and they touch the save format, so do them early.
- Battle readability (HP bars, shots) is worth more than walls or a 4th troop, and it is cheap.
- Tutorial and phone layout (T5) block testers; art does not. T5 must not sit behind a big art pass. Do the art pass as a time-boxed "replace the capsules and emoji" step, with asset sourcing done by a helper in parallel.
- Content (walls, Trebuchet) after PvP is right, but Trebuchet belongs in the balance pass.

Minimum set before testers: refilling, Keep-scaled camps; readable battles; balance fixes; Anki link; lean async PvP (raid, shield, defense log, revenge, trophy number); move buildings; tutorial plus phone layout; art that doesn't look like capsules; private link and kit. Walls are "strongly wanted", not minimum.

## 4. Three biggest risks to the playtest itself

1. Empty raid list (not "too few online": async raids don't need anyone online). With 10 players, 48-hour new-player shields and 8-hour shields, at many moments nobody is raidable, and levels will spread widely. Mitigations:
   - Generated camps scaled to your Keep level, refilling daily (or every few hours). Always at least 3 targets.
   - Raiding a shielded player's snapshot as "practice" (no loot taken from them, small loot from the camp pool).
   - Shields shortened for the test: 24 hours for new players, 4 hours after a raid.
   - Seed 3-5 bot villages from scripted saves.
   - Check: a test that, for a 10-player state with everyone shielded, every player still has at least 3 targets with loot.
2. Balance breaks in week 1 (spam wins, active tapping races ahead, study boosts dwarf or vanish next to PvP loot). Mitigations:
   - An economy playthrough sim: a scripted player with 3 sessions a day and 300 reviews reaches about Keep 4-5 by day 3 and is not maxed by day 7. A no-study player is behind but not stuck.
   - A battle test: a level-matched army against a level-matched village gives 1-2 stars, not 3.
   - Building HP by level, Trebuchet splash, loot 10% per SPEC-3, mines holding hours.
   - A small server-side tuning table (loot %, shield hours, camp loot, chest pay) read at start, so you can retune mid-test without a rebuild. About half a day.
3. Retention and Anki friction. Phone-only reviewers count only after a desktop sync (the honour button was dropped). The web game can't send push notifications. Day-2 return depends on a reason to come back. Mitigations:
   - Screen testers for desktop Anki that syncs daily; the guide says so plainly.
   - A daily "you were raided / your camps refilled / chest waiting" summary on open.
   - Conner posts one daily nudge in the tester group chat (his send, not automated).
   - Success lines already set (5 of 10 active on day 7). Add a day-2 return check and decide early.

## 5. Revised order with rough effort (agent working days)

1. Economy and combat fix pass (1.5 d). Mines hold hours of production. Building HP scales with level. Training queue up to housing, or a 2nd Barracks. Loot 10% (reconcile plans_from_town with SPEC-3). Camps refill daily, plus 4-6 generated camps scaled to your Keep. Add an economy playthrough sim test and a level-matched battle test.
2. Battle readability (1 d). HP bars on damaged buildings and troops, shot tracers from defenses, hit flash, loot pop-up. Free CC0 sounds if cheap.
3. Anki link T4 per ANTICHEAT.md (4 d, time-boxed).
4. Move buildings (0.5-1 d). Needed before PvP defense means anything.
5. Lean async PvP T7 (3-4 d). Raid snapshot, raid RPC with loot cap and shield, defense log (text), revenge, trophy number plus top-10 list, practice raids on shielded players, a tuning table. No leagues, no replays.
6. Trebuchet with splash (0.5 d). The answer to spam.
7. Tutorial plus phone layout T5 (1.5-2 d).
8. Art pass, time-boxed (3-4 d; asset sourcing by a helper in parallel from step 3 on). One model per building, real troop models, icons, logo.
9. Walls, simple (2-3 d, stretch). Wall pieces; troops hit a wall that blocks their straight line; wall levels as a gold sink. Cut first if late.
10. Private link, tester kit, survey T6 (1 d), verifier on the deployed link, then the owner's final go.

Total: about 18-22 working days with walls, about 16-19 without.

## Verdict

Agree with changes. The draft's Anki-first instinct is right, but it misses that the core loop runs dry after the first session (non-refilling camps, 5-minute mines) and that combat scaling is broken (flat building HP, no troop levels, a 480-unit army, only single-target defenses). Fix those cheaply before PvP. Cut leagues, obstacles, the streak builder and replays. Keep computer camps as the main raid supply. Move tutorial and phone ahead of a time-boxed art pass.
