# Art review of ROADMAP-DRAFT.md step 4 (skeptical art director pass, 2026-11-28)

Scope: the art plan for Stonehaven (generic kingdom, Clash-like 3D base builder, Exact engine, web build), $0, free tools only. Tags: CONFIRMED (file:line or URL) or UNVERIFIED.

## Verdict on the draft

Direction is right (CC0 kits plus free tools), but as written step 4 will produce an inconsistent, placeholder-feeling result. Five problems:

1. **It mixes artists.** "Kenney castle kits for buildings + Kenney Mini Characters / Quaternius for troops" puts three style languages side by side (Kenney flat colours, Quaternius smooth PBR-ish, a third character proportion). Mixing kits is the main reason free-asset games look cheap. There is a single-artist CC0 set that covers almost everything: KayKit (Kay Lousberg). Use it and drop Kenney meshes.
2. **"Vinyl toy" isn't available for free.** No CC0 kit is smooth, glossy vinyl. KayKit is the closest: chunky, big-head chibi characters, rounded-but-faceted low poly, one gradient atlas. The owner should knowingly accept "chunky toy low-poly" for the playtest. (DESIGN-DIRECTION.md's style pick was made for the med-school theme, which has since been replaced by Stonehaven.)
3. **AI-generated icons won't match the 3D.** Canva/Figma images come out painterly or glossy 2D in a different light and palette. Make HUD icons by rendering the game's own 3D models (Blender, free, scripted). Keep Canva/Figma for the logo backdrop and marketing art only.
4. **"3 visual tiers" has no plan behind it.** No free kit ships 8 levels, or even 3, of the same building. Tiers have to be kitbashed from parts of the same kit (see below). This is code work, not downloading.
5. **Ordering hides the engine risk.** Art comes after PvP and the completeness pass, so skinned-troop size, bake refusals and load time would surface late. Run a 1-day art spike (Keep + Mint + one animated troop + one defense, through the real bake, on the web) before PvP. Keep the full pass where it is.

The draft also has no effort estimate, no size budget and no consistency checks. Those are below.

## What the engine can and can't do (verified)

- **Formats:** glTF/GLB only, baked at build time to `.model`/`.tex`/`.sound`. No FBX or OBJ at runtime. CONFIRMED `.exact/checkout/game/bake/README.md:3-5`, `.exact/checkout/game/recipes/blender-export.md:3`.
- **Accepted:** triangles with POSITION, NORMAL, TANGENT, TEXCOORD_0, COLOR_0, JOINTS_0/WEIGHTS_0 (skins), PNG/JPEG textures, skeletal animation, unlit, emissive strength, texture transform, clearcoat/sheen factors. **Refused:** Draco, morph targets (no blendshape faces), sparse accessors, second UV set, and other extensions. CONFIRMED `blender-export.md:22-28`. Meshopt: the docs disagree. bake/README.md:129 says meshopt and quantization are accepted; blender-export.md:26 says meshopt is refused. Treat as UNVERIFIED until the spike bakes one file (and file the doc conflict with `bun exact.mjs feedback note`).
- **Skinned, animated characters:** supported. Clips by name, `Animator` transitions that keep phase, additive or masked layers (a hit flinch over a walk), markers (footsteps, attack-impact frames), sockets for weapons. GPU skinning with per-instance culling, so a far crowd skins nothing. CONFIRMED `.exact/checkout/game/engine/README.md:271-294`, `.exact/checkout/game/render/README.md:316,1068-1078`. A working Fox example exists: `.exact/checkout/game/games/skinned-fixture/art/fox.glb`.
- **Fallback with no art:** `exact_game::rig` builds skinned humanoids from capsules with walk/run/idle/flinch clips in code. CONFIRMED `engine/README.md:327-352`. Useful as a stand-in, but it's still capsules, so it isn't the answer.
- **Toon look:** per-mesh ink outlines and rim light are read from glTF extras. CONFIRMED `render/README.md:675-688`. The repo already has `village/tools/add_outline.py`, which applies them (LOOK-PLAN.md). Rim light on everything plus outlines is the cheapest way to make mixed parts read as one style.
- **Team colours and tier tints at runtime:** `MaterialOverrides`, `NodeMaterials`, per-instance texture swaps, Glow, emissive. CONFIRMED `.exact/checkout/game/README.md:516-560` (table rows "Team colours", "A skin or theme").
- **Textures:** RGBA8 at most 2048², block-compressed (BC7/ASTC) up to 4096². Identical textures are deduplicated by content, so a shared atlas downloads once. CONFIRMED `bake/README.md:17,34,45-50`. A colour texture referenced by URI must live in `art/textures/` (the existing `village/tools/fix_glb_uri.py` handles this). CONFIRMED `bake/README.md:78-82`.
- **Load-time rule:** a model a tick animates or reads must be in `Game::ASSETS`, and setup waits for those. Everything else can go in `STREAMED`. CONFIRMED `game/README.md:1127-1134`. Today the game streams all models (`village/logic/src/lib.rs:283`). Animated troops would move into ASSETS, so **troop file size directly delays first play**.
- **Particles / VFX:** emitters with textured flipbook atlases, stretch (sparks, arrow streaks), additive blending, soft ground fade, world-space trails. CONFIRMED `engine/README.md:296-311`, `game/README.md` table row "Fire, smoke and sparks".
- **Sound:** WAV or Ogg Vorbis under `art/`, baked to `.sound`, streamable. Synthesized sounds are also possible. CONFIRMED `bake/README.md:84-92`, `engine/README.md:312-324`. `app.json` needs `game.audio`, which it doesn't have today (`village/app.json`). The exact key shape is UNVERIFIED.
- **HUD:** Contract can show PNG images (`image "assets/x.png"`) and bundle a font (`font "Family" = "path.ttf"`). CONFIRMED `.exact/checkout/docs/contract-for-agents.md:368,2781`. Today the HUD uses emoji (`village/app.contract:198-200`).
- **Hard size limits:** none found for whole downloads. Per-file limits are 2048² RGBA8 textures, a 256 MiB decoded meshopt view and 32 MiB PCM per sound (`bake/README.md:17,89,139`). The real limit is first-load time on a phone, which we set ourselves (budget below).
- **How art is loaded today:** 10 Kenney Castle Kit GLBs plus one colormap (`village/art/`, 372 KB), listed in `MODELS` (`village/logic/src/lib.rs:817-826`) and stacked into buildings in code (`lib.rs:838-855`). Troops are coloured capsules (`village/logic/src/fight.rs:171`).

## The free kit that fits: KayKit (one artist, CC0)

All of these are CC0, "name your own price" ($0 works), glTF included, and share one gradient-atlas texture style:

| Pack | Free contents relevant to us | Evidence |
|---|---|---|
| Medieval Hexagon | 200+ models. Buildings in 4 team colours (blue/red/green/yellow): castle, barracks, archery range, mine, windmill (separate fan node), watermill, market, tavern, church, blacksmith, lumbermill, well, homes, tower A/B, tower base, **tower_catapult (separate turret + arm nodes)**, walls (straight, gate, inside/outside corners), stone and wood fences, flags in 4 colours, **building_scaffolding, building_stage_A/B/C (construction stages), building_destroyed (rubble)**, trees, rocks, hills, mountains, clouds, crates, barrels, sacks, tent, target, wheelbarrow, projectile_catapult. Every building samples one 16 KB atlas, `hexagons_medieval.png`. Building sizes: castle 5.7k tris and 2 m wide, barracks 4k, mine 1.1k. | CONFIRMED file tree via GitHub API, repo KayKit-Game-Assets/KayKit-Medieval-Hexagon-Pack-1.0 (LICENSE.txt CC0). Tris and bounds read from the glTF JSON. https://kaylousberg.itch.io/kaykit-medieval-hexagon |
| Character Pack: Adventurers | 5 rigged characters (Knight, Barbarian, Mage, Rogue, Rogue_Hooded) plus props (swords, axes, crossbows, shields, arrows, quiver, staff). Knight.glb: 1 skin, 41 joints, 6.9k tris, no extensions, no morphs, linear keys, **76 clips embedded** (walk A/B/C, run, 1H/2H melee attacks, 1H/2H ranged shoot, hit A/B, death A/B, cheer, idle, block…). About 3.6 MB per GLB. | CONFIRMED by reading the GLB JSON chunk from GitHub repo KayKit-Character-Pack-Adventures-1.0. https://kaylousberg.itch.io/kaykit-adventurers |
| Character Pack: Skeletons | 4 animated skeletons (about 4.8 MB each). Optional enemy or "raider" flavour for computer bases. | CONFIRMED GitHub tree, https://kaylousberg.itch.io/kaykit-skeletons |
| Character Animations | 150+ clips for the KayKit rig, free. | CONFIRMED https://kaylousberg.itch.io/kaykit-character-animations |
| Fantasy Weapons Bits | 25+ weapons free, including bows and spears (for Spearman, Bowman, Lancer). Which items are in the free tier is UNVERIFIED. | https://kaylousberg.itch.io/fantasy-weapons-bits |
| Resource Bits | 75+ free resource models including gold. Gems are paid; grain is not listed (UNVERIFIED). | https://kaylousberg.itch.io/resource-bits |
| Forest Nature Pack | trees, rocks, grass | https://kaylousberg.itch.io/ (listing) |

Style match: chunky, big-head, toy-like. It fits "chunky big-head characters" well and "vinyl toy buildings" only partly (faceted, not glossy). It isn't Supercell's look: no moustached barbarian, no pink potions, generic medieval. Note that KayKit's own "Barbarian" is a bearded brute, so don't use it as a Clash-barbarian lookalike. Use the Knight and Rogue as the base troops.

Kenney (Castle Kit CC0, 75 files, https://kenney.nl/assets/castle-kit; Fantasy Town Kit CC0 per opengameart.org/content/fantasy-town-kit) and Quaternius (Medieval Village MegaKit, CC0 but the free tier is a subset, https://quaternius.com/packs/medievalvillagemegakit.html) are fine kits, but each is a different artist. Use them only for a part KayKit lacks, and retexture it to the KayKit atlas palette.

Gaps in KayKit free: no ballista, no battering ram, no horse (Lancer can't be mounted), no grain or crown props. Fixes are in the table below.

## Piece-by-piece plan

Effort is in agent-days of build plus check, sized for a 10-tester playtest.

| Piece | Source and method | Effort |
|---|---|---|
| **Pipeline** (do first) | One script (`tools/import_kaykit.*`): glTF → GLB with the texture URI moved into `art/textures/` (extend fix_glb_uri.py) → strip unused clips and unused held-item meshes → run add_outline.py (outline + rim) → bake. Clip and mesh stripping uses free tools: the glTF-Transform JS API (MIT) or headless Blender. Add a size-budget test and the blender-export.md `export_problems` test over all shipped models. | 1 day |
| **Buildings, 10 types × 3 tiers** | Base: the blue variant for the player, red for enemy bases (free readability win). Kitbash tiers in code from KayKit parts, the way `lib.rs:838-855` already stacks Kenney parts. Lv1-3: base model on a stone pad. Lv4-6: the same model plus fence ring, flags, crates/sacks, a taller roof part. Lv7-8: a bigger silhouette swap plus a gold-trim material override and a soft glow. Mapping: Keep = home_B → tower_A + walls → castle. Mason's Lodge = lumbermill/blacksmith. Mint = mine (+ gold pile). Treasury = market (+ chests and sacks that fill with stored gold). Farm = windmill (fan node spins). Granary = watermill or home + sacks. Barracks = barracks. Muster Ground = tents + target + wood fence. Watchtower = tower_B (+ archer on top later). Ballista = tower_base + scaled crossbow_2handed (kitbash). Trebuchet (step 3) = tower_catapult (turret and arm nodes aim and fire). | 2 days |
| **Construction and destruction** | building_scaffolding / stage_A-C while a timer runs; building_destroyed rubble when a building is razed in battle. Both ship in the pack. | 0.5 day |
| **Walls** | wall_straight / corner pieces picked by neighbour mask (auto-connect). Tiers: wood fence → stone fence → castle wall. | 1 day |
| **Troops** (Spearman, Bowman, Lancer, + Ram) | Spearman = Knight + spear (Weapons Bits) + shield. Bowman = Rogue_Hooded + bow (or the pack's crossbow). Lancer = armoured Knight variant with a long spear and a red cloak tint, on foot. One line: "Lancer" implies a horse; renaming it is the owner's call, otherwise keep it on foot. Ram = kitbash in code from lumber + wheelbarrow wheels + a crate roof, carried by 2 small figures or wheeled. Keep 5 clips per troop: Idle, Walking, attack (1H melee or 2H ranged shoot), Hit, Death. Drive them with `Animator`, and fire damage on the attack clip's marker so the hit lines up with the swing. Projectiles: arrow.gltf, projectile_catapult.gltf. | 2 days |
| **Terrain** | Square grid, so skip the hex tiles. A grass plane with a subtle tiling texture (`art/textures/`, generated or CC0), a dirt rim (already there), and a KayKit forest/hills/mountains ring outside the play area (replaces the Kenney trees and rocks in `lib.rs:862-878`). Clouds drifting on the horizon. | 0.5 day |
| **VFX** | Engine emitters. Dust puff on build, place and destroy; sparks on hit; arrow streaks (stretch); a smoke column on a razed building; a coin and grain "pop" on collect; a white hit flash via Glow. Textures from Kenney Particle Pack (CC0, 80 files, https://kenney.nl/assets/particle-pack); its exact contents are UNVERIFIED. Battle juice reads as "good-looking" more than building detail does. | 1 day |
| **HUD icons** | Render Gold (coins pile), Grain (sack or wheat), Crowns (crown kitbash), builder, troops and buildings from the actual models with an orthographic Blender camera, the same light and an outline pass, then export PNGs. They match the 3D because they are the 3D. Replace the emoji in `app.contract:198-200`. | 1 day |
| **UI frame and font** | Contract only: chunky rounded panels in a wood/stone palette sampled from the KayKit atlas, plus a bundled display font. Lilita One or Luckiest Guy from Google Fonts; their OFL licence is UNVERIFIED in this pass, so check it before bundling. Already partly styled (LOOK-PLAN step 4). | 0.5 day |
| **Logo** | Wordmark in the display font with an outline + drop shadow, plus a simple shield/keep emblem rendered from the castle model. Canva/Figma generation can make the backdrop or splash. Avoid Clash's gold-on-wood serif shield lockup. | 0.5 day |
| **Sounds** | Kenney CC0 audio: RPG Audio (50 files, https://kenney.nl/assets/rpg-audio), plus Impact, Interface and Music Jingles packs (UNVERIFIED contents, same CC0 licence family). About 15 cues: tap, place, build start/finish, collect gold/grain, train, deploy, sword hit, arrow, catapult, building destroyed, star earned, win, lose, chest open. Add `game.audio` to app.json. Music: one CC0 loop (source UNVERIFIED) or none for the playtest. | 0.5 day |
| **Art spike** (before PvP) | Keep + Mint + Spearman walking and attacking + Watchtower through the real pipeline on the web build: bake accepted, load time measured, screenshot. | 1 day (counts toward the above) |

**Total: about 9-10 agent-days**, including the 1-day spike. Most of it is code kitbashing and wiring, not downloading.

## Size budget (make it a test)

- Buildings: tiny. One 16 KB shared atlas for all of them. Each glTF is a few k triangles.
- Troops are the risk: about 3.6 MB per Adventurers GLB because 76 clips are embedded. Stripped to 5 clips and one weapon, they should drop well under 0.5 MB each (estimate, UNVERIFIED until the spike measures it). Unstripped, 4 troops would add about 15 MB that setup must wait for (troops must be in ASSETS).
- Proposed budget: total `assets/` gzipped ≤ 6 MB, ASSETS (blocking) ≤ 2 MB, first play on a mid phone over 4G ≤ 5 s. Enforce with a test that sums the baked files.

## Consistency risks and the checks that prevent them

| Risk | Check that catches it |
|---|---|
| Mixed artists | A test that every shipped model's base-colour texture is one of the KayKit atlases (by content digest), with an explicit allowlist for any exception. |
| Scale drift (a 2 m KayKit castle next to a stretched kitbash) | A per-kind footprint table plus a test that each building's baked bounds fit its tile footprint within ±10%. |
| Every model maps to a real file, all tiers | A test over KINDS × level 1..8 asserting that the model list resolves in `presentation_models()`. |
| Export defects (double-sided walls, black vertex colours) | The `export_problems` test from `blender-export.md:30-48` over all models. |
| Lighting mismatch between icons and world | Icons rendered from the same models, with the sun direction and colour copied from `lib.rs:345` in the Blender script. |
| Team-colour confusion | Player = blue and enemy = red, everywhere. Screenshot check in battle. |
| Supercell look-alike | A reviewer agent compares the screenshots to a short "don't" list (barbarian moustache, pink elixir, gold-on-wood shield logo, Clash HUD corners). |
| "Looks good" is subjective | A fixed screenshot set (village lv1, village lv8, battle mid-fight, phone 390×844) taken by `bun exact.mjs agent web … screenshot`. A fresh reviewer scores them against a one-page style sheet (palette, outline width, light) before testers see them. |

## Good enough for a 10-tester playtest vs later polish

**Playtest bar:**

- Every building, troop and defense is a real model from one kit with a distinct silhouette, and no capsules or emoji remain.
- 3 visible tiers per building, so upgrades feel rewarding.
- Scaffolding while building and rubble when destroyed.
- Troops walk, attack (with the impact timed to the swing) and die.
- Arrows and catapult stones fly.
- Hit flash, dust and sparks.
- Rendered resource icons, a custom font and a chunky frame.
- A wordmark logo.
- About 15 sound cues.
- First play in 5 s or less on a phone.

**Later polish (after the playtest says the game is worth it):**

- A unique look for each of the 8 levels.
- A true glossy "vinyl" pass (smooth shading and bevels in Blender, or a commissioned set).
- Mounted Lancer, a hero model, custom faces.
- Building idle animations (smoke, flags, workers).
- Music, a splash/loading screen, store art.
- A paid KayKit Extra tier only if money is ever approved: the Hexagon Extra is listed at $4.99-9.99 and includes units and horses.

## Simpler alternative considered

Keep the current Kenney Castle Kit for buildings, add outlines and a palette, and use `exact_game::rig` capsule humanoids for troops (no downloads, about 3 days). Rejected: rig troops are still capsules (the "plain dots" complaint in STATE.md:79), and the Kenney kit has no construction, rubble, team colours or characters. It saves about 5 days but misses "testers find it good-looking".
