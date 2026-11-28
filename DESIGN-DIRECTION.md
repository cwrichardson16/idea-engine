# Design direction (v2 after 2 reviewers, 2026-11-28)

## Changes from the review (these override the sections below)
- **Study is a boost, never a gate.** Raids always work on a base energy. Studying adds "rested" bonuses: double loot, Knowledge-only unlocks, streak shields. Knowledge and energy are never sold.
- **Backlog siege is the core** (V1.md decision V2): overdue cards arrive as enemy waves sized by your backlog; clearing reviews weakens them. Quiz defense is dropped: it rewarded accuracy, which DESIGN.md forbids (grades are self-reported). Rewards come from review counts and time, never grades.
- **Order:** battles → study loop + siege (web, fake review button) → concept art → 10-person med-student playtest (7-day return, reviews/day) → multiplayer only if the playtest is good.
- **Money:** sell from our own site (membership + cosmetics), not through AnkiWeb; never "Anki" in the name.
- **Supercell distance, before anything public:** new title (no "Clash of"); retune every number (our SPEC-2 troop/building stats match Clash level 1 almost exactly); themed resource and building names (no Elixir/Gold Mine bundle, no gems); themed enemy instead of goblin camps; our own HUD layout; make the "Town Hall star" our own core building. Mechanics themselves are fine.
- **Theme lean:** A (med school), reachable niche, impossible to confuse with Clash; B's immune-cell ideas can become a wing of it.


Conner wants this to be a game he could monetize, with no reason for Supercell to come after us, while not being over-cautious. Today the build is a working Clash-style base builder (village, builders, resources, army, battles vs computer camps in progress) in the Exact engine, with placeholder Kenney medieval art. Plans: V1.md, SPEC-2.md, STATE.md.

## 1. What makes this OUR game (the hook)
Studying powers the village. Every flashcard review you do (Anki or built-in) earns Knowledge, the key resource:
- Knowledge finishes timers early (already built), and later unlocks things that can't be bought any other way (research, special troops/spells, defenses).
- The "energy" that limits raids is earned by studying, not bought, so the game rewards studying and can't be paid past.
- Streaks and daily study goals give shields, bonuses and cosmetic rewards.
Pitch: "the base-builder that makes you study." The genre is crowded; the study loop is the moat.

## 2. Theme options (pick one; all original)
- **A. Med-school humor (the original plan):** your base is a teaching hospital/campus. Resources: Coffee and Grant Money. Troops: Interns, Residents, a hulking Attending. Defenses: Pagers, a Malpractice Cannon. Raids on rival programs. Audience: med students (heavy Anki users). Risk: narrow audience; jokes must land.
- **B. Body defense:** your base is a body region; immune cells defend; you raid infections/tumors. Cartoon biology, genuinely educational. Distinct look (no castles). Risk: "raid" fiction feels odd; PvP between bodies is weird.
- **C. Wizard academy:** a school for mages; studying = learning spells; resources: Mana and Ink; troops: apprentices, golems, familiars. Broad audience, fits "study = power" naturally. Risk: closest to generic fantasy, so closer to Clash's world.
- **D. Cozy critter study town:** animal students build a town; raids are playful pranks/heists, not war. Wider, younger, friendlier audience. Risk: softer combat feel.

## 3. Gameplay changes that make it ours (beyond reskin)
- Knowledge-gated progression (above).
- Raid energy from study (no gem-skip of raids).
- "Quiz defense" idea: when attacked, base gets a defense bonus from your recent study accuracy (async: computed from stats at snapshot time).
- Clans become study groups: shared streak goals instead of only troop donations.
- Out: copying Clash's exact numbers tables, troop roster, building list one-for-one.

## 4. Multiplayer shape
Async raids (attack a saved snapshot of another player's base), like the genre standard. Deterministic battle sim already built for it: the server replays a raid's drop log to verify results. Computer camps use the same base format, so swapping camps for real players is a data change. Small hosted server later (ask Conner before anything live or paid).

## 5. Monetization (first thoughts)
Cosmetics (skins, base themes), a season pass of cosmetic/study rewards, maybe gems for builder time. Never sell Knowledge or raid energy (keeps "studying is the only way" honest).

## 6. Staying clear of Supercell (guardrails)
- Name: drop "Clash of ___". New name before anything public.
- Our own art, characters, names, sounds, icons, HUD layout. No look-alike characters (moustached barbarian, hog rider, elixir-pink potions, etc.).
- Mechanics (base building, timers, troops, async raids, stars) are genre conventions and fine to use.
- Our own numbers and roster, not their tables.
- "Works with Anki", never Anki in our name/branding.
- Before launch with money involved: a short trademark search/lawyer check on the name.

## 7. Order
Battles (now) → multiplayer shape → Anki/study loop → theme + art (concept images to pick a style, then AI 3D or an artist).
