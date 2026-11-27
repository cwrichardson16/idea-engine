# SPEC: Slice 2, army and battles (Clash of C____)

Status: approved after 2 pressure-test rounds (no blockers; 7 gaps plus 2 contradictions fixed). Slice 1 is done (SPEC.md, STATE.md).

## Goal
You can train a small army in your village, attack computer-made goblin camps, watch your troops fight their defenses, win stars and loot, and come home richer. It looks like a game: a tilted camera you can drag and zoom, and buildings that read as buildings, not boxes.

## Finish line (all must hold, with evidence)
1. `bun exact.mjs test-rust` passes, including the new battle tests below, plus the determinism lints. `bun exact.mjs test web` passes with a rewritten `app.test.contract` covering the village, battle and result screens.
2. `bun exact.mjs contract build app.contract --json` gives `[]`.
3. In Chrome, a full loop with screenshots:
   - build a barracks and an army camp, then train troops;
   - Attack, choose camp 1, deploy troops, watch the fight, and see the result screen (stars, %, loot);
   - home again: loot is in storage, the troops used are gone, and camp 1 shows its stars;
   - reload mid-training: the army and training progress persist.
4. Two reviewers: a skeptic on the diff, and a verifier on the Chrome flow. Their confirmed issues are fixed.
5. Commit and push to the feature branch (never main). STATE.md is updated.

## Choices made (Conner can override)
- **Troops (3):**

  | Troop | Housing | Training cost and time | Behaviour |
  |---|---|---|---|
  | Fighter | 1 | 25 elixir, 5 s | melee, goes for the nearest building |
  | Archer | 1 | 50 elixir, 8 s | ranged (3.5 tiles), goes for the nearest building |
  | Brute | 5 | 250 elixir, 20 s | slow and tanky, goes only for defenses (falls back to any building once no defenses are left) |

  These are V1.md's troops. The med-humor names come at reskin time.
- **New buildings:**

  | Building | Unlocks at | What it does |
  |---|---|---|
  | Barracks | TH1 | trains one troop at a time from a queue of up to 10. Level 2 unlocks Archer, level 3 Brute. |
  | Army Camp | TH1 | holds 20 / 30 / 40 housing by level |
  | Cannon | TH1 | ground only, range 7, single target |
  | Archer Tower | TH2 | range 8, single target, faster but weaker |

  Training keeps going while you're away, the same as production.
- **Battles are against computer camps only** (no server). There are 4 hand-made goblin camps:
  - Camp 1 is the tutorial: no defenses.
  - Later camps add cannons, then archer towers, then more of both.
  - Each camp holds a fixed amount of loot.
- **Battle rules:**
  - 3 minutes.
  - Tap outside the red border to drop the selected troop.
  - It ends when time runs out, every deployed troop is dead and none are left to drop, or the camp is fully destroyed. You can also press Surrender.
  - Stars: 50% destroyed, the Town Hall destroyed, and 100% destroyed. V1.md said "no stars yet"; stars are cheap and they give the campaign a goal, so they're in.
  - Loot comes out of a building as it is destroyed. It's credited at the end and capped by your storage.
  - Troops you deploy are used up.
  - A camp can be replayed. Loot only comes the first time, and stars keep your best result.
- **Battle simulation:**
  - It is deterministic, on the same 30 Hz tick, and lives in pure Rust (`battle.rs`) with whole-number maths. That means the same drops always give the same result, and it can be tested.
  - Troops walk in a straight line to their target. There are no walls, so no path-finding.
  - Defenses fire at the nearest troop in range.
- **Look pass:**
  - Tilted camera: drag to pan, mouse wheel or pinch to zoom, kept inside the map. A tap still means movement under 12 px.
  - Buildings use Kenney CC0 models (Castle, Fantasy Town and Nature kits; Conner approved the download on 2026-11-27). The fallback is hand-made shapes from the engine's cuboid, cylinder, cone and capsule meshes:
    - Town Hall: keep, roof and flag
    - mine: pit and cart
    - towers: tall, with a top
    - cannon: barrel
    - troops: small capsules with a colour per type
  - A few trees and rocks go around the edge.
- **Saving:**
  - Save version 2 adds the army, the training queue and the campaign stars.
  - Version 1 saves still load, with an empty army.
  - A battle is not saved. If you reload mid-battle, the troops you deployed are lost and you get no loot (this matches Clash, where leaving counts as the end).

## Build order: two phases, each with its own commit, checks and verifier
- **Phase A:** camera, look (Kenney CC0 models, hand-made shapes as fallback), army buildings and training, home defenses, save v2.
- **Phase B:** battle sim, campaign, battle and result screens.

## Numbers (first pass; tune later)

| Building | Size | How many (TH1/2/3) | Cost (L1/L2/L3) | Build time | Notes |
|---|---|---|---|---|---|
| Barracks | 3 | 1/1/1 | 200/600/1500 elixir | 20/60/120 s | L2 unlocks Archer, L3 Brute |
| Army Camp | 4 | 1/1/2 | 250/750/1500 elixir | 20/60/120 s | housing 20/30/40 |
| Cannon | 3 | 2/2/3 | 250/700/1500 gold | 15/45/90 s | range 7, damage 9/12/16 every 0.8 s, ground |
| Archer Tower | 3 | 0/1/2 | 1000/2000/3000 gold | 60/120/180 s | range 8, damage 6/8/10 every 0.5 s |

| Troop | Hit points | Damage per second | Speed (tiles/s) | Range | Targets |
|---|---|---|---|---|---|
| Fighter | 45 | 8 | 1.6 | 0.6 | nearest building |
| Archer | 20 | 7 | 1.6 | 3.5 | nearest building |
| Brute | 300 | 11 | 1.0 | 0.6 | defenses first, then anything |

Camp building hit points: Town Hall 400, defenses 300, storages 250, mines/collectors 200.

## Rules
- **Housing** counts trained AND queued troops. You can't queue past it ("Army camps full").
- **Cancelling** a queued troop refunds it in full.
- **Training snapshot:** the army is snapshotted when a battle starts. Troops that finish training mid-battle can't be deployed in that battle.
- **Destruction %** = buildings destroyed ÷ all buildings, counting the Town Hall.
- **Loot** sits in the camp's Town Hall, storages, mines and collectors. Each one drops its share when destroyed. It's credited at the end, capped by your storage, and anything over the cap is lost.
- **Replays:** each camp remembers which buildings were already looted. On a replay only un-looted buildings give loot. Stars keep your best.
- **Battle clock** counts ticks (5400 = 3 min), never village time. The world reports busy during a battle.
- **Drag** pans the camera. Moving buildings is out of scope.
- **Save v2:**
  - New building kinds are only ever added at the END of `KINDS`.
  - A literal v1 save string, captured before any change, stays a fixed test.
  - The old "v2 is refused" test is rewritten on purpose.
  - `SaveContract` gets bumped.
- **Dev-only buttons:** "Dev: +5000" (gold and elixir) and "Dev: TH+1". They exist so the verifier can reach Town Hall 2–3 content. They are hidden behind a dev flag at reskin.

## Engine facts (checked in ~/Projects/exact source)
- **Drag:** `input.pointer()` gives `.delta` per tick. **Zoom:** `input.wheel().y`. A trackpad pinch arrives as wheel. See game/recipes/orbit-camera.md.
- **Models:**
  - Put the `.glb` in `village/art/`, add `village/app.json` with `{"game":{"assets":true}}`, then use `Mesh::asset("<name>.model")` and list the name in `Game::STREAMED`.
  - Tests use `sim_with_art!`.
  - Turning assets on means one fresh module compile.
- **Tweens:** scalar `Tween` and `Spring`; move the transforms each tick.
- **Picking:** keep our own ray (`tile_at`) and make it use the moving camera.

## Feature checklist
- [ ] **G1 Camera**
  - Check: drag pans and wheel zooms, both kept inside the bounds; a tap still selects; picking stays right after pan and zoom.
  - Proof: a test of `tile_at` with an offset camera; Chrome screenshots zoomed in and out.
- [ ] **G2 Look**
  - Check: each building kind has a distinct shape; trees and rocks are present.
  - Proof: a Chrome screenshot.
- [ ] **G3 Army buildings**
  - Check: barracks and army camp in the shop with TH caps; the training queue costs elixir, takes time, keeps to the housing limit, and finishes while away; troop unlocks follow barracks level.
  - Proof: tests, plus a Chrome screenshot of the training panel.
- [ ] **G4 Defenses at home**
  - Check: cannon and archer tower can be built and upgraded.
  - Proof: tests and a screenshot.
- [ ] **G5 Battle sim**
  - Check:
    - a fighter destroys an undefended camp;
    - a cannon kills fighters in range and ignores those out of range;
    - a brute ignores non-defenses until the defenses are gone;
    - stars and % are correct;
    - loot equals the sum from destroyed buildings, capped by storage;
    - the same drops give the same result (determinism);
    - the battle ends on time, when all troops are dead, or at 100%.
    - no two troops dropped on the same tile share a position after 30 ticks (spread);
    - a battle always ends within 5400 ticks (no wall-clock timing test);
    - the battle clock ignores `away`.
  - Proof: `tests/battle.rs`.
- [ ] **G6 Battle UI**
  - Check: Attack shows the campaign map with stars; in battle: troop picker with counts, timer, %, stars so far, Surrender; deploying inside the red border is refused with a reason; the result screen leads home.
    - every published HUD number is finite and ≥ 0;
    - "skip gems" during a battle changes nothing.
  - Proof: Chrome screenshots plus tests.
- [ ] **G7 Save v2**
  - Check: army, queue and stars round-trip; a v1 save loads; damaged saves are rejected.
  - Proof: tests and a Chrome reload.

## What the player must NOT see
- Troops walking through each other into one blob. They need a small spread.
- Negative or NaN numbers, troops dropped inside the border, or loot beyond storage.
- A battle that never ends.
- Camera dragged to an empty void.
- Their army doubled or lost on reload.
- Gems spent by a stray tap.

## Out of scope
Walls, traps, spells, heroes, the lab, trophies, PvP, clans, a server, real money, the Anki hookup, sound, troop animation beyond moving and attacking, camps that defend themselves when raided.

## Premortem: this fails if…
1. **The battle sim gets slow or messy.**
   - Fix: whole-number maths; at most 80 troops (2 camps × 40 housing) and 30 buildings; the target search is a simple loop.
   - Check: a battle-end tick bound, plus the full suite staying fast.
2. **Drag to pan breaks tap to select and deploy.**
   - Fix: keep the 12 px tap rule.
   - Check: a Chrome test of drag then tap; a unit test that drag deltas don't count as taps.
3. **Battle state leaks into the save, or a reload duplicates the army.**
   - Fix: troops are deducted from the army the moment they're dropped; the battle itself is never saved.
   - Check: a test that saves mid-battle, reloads, and finds the army reduced and no loot.
4. **The rebuild loop eats the time** (dev rebuilds take 7 to 45 minutes).
   - Fix: test-first in Rust; run the Chrome checks in batches, not after every change.
5. **The model pipeline is harder than expected.**
   - Fix: turn assets on as the very first step of Phase A. If a model isn't rendering in Chrome after one rebuild, use the hand-made shapes and move on; models come back later.
