# Roadmap draft: from here to "good art + testers playing a robust Clash-like game"

Owner goal (2026-11-28): the game looks good (real art, not placeholders) and ~10 testers play a robust Clash-of-Clans-like game, with real Anki reviews boosting them. $0 budget (free tools only: Canva/Figma generation, CC0 kits, no paid APIs). Never share the link until the owner's final go.

## Where we are (facts, see STATE.md)
- Engine: Exact (Rust world + Contract HUD), web build works (1.5 MB gzip wasm), WebGPU renderer.
- Built: village building with 2 builders, resources + storage, 8 levels per building with timers, army training, battles vs 4 computer bases (deterministic sim, stars, loot), study rewards (Knowledge, chests, streak, power hours), cloud save + restore codes + bug report (Supabase free tier), dev buttons hidden.
- Names: generic kingdom set "Stonehaven" (Gold, Grain, Crowns, Keep, Mason's Lodge, Mint, Farm, Barracks, Muster Ground, Ballista, Watchtower, Spearman, Bowman, Lancer).
- Art: placeholder low-poly (Kenney-style CC0 medieval pieces + primitive capsules for troops), emoji icons in the HUD. Earlier picked style: rounded "vinyl toy" buildings + chunky big-head characters, warm light (DESIGN-DIRECTION.md).
- Approved anti-cheat Anki design (ANTICHEAT.md), ~4 days to build.

## Proposed order
1. **Anki link (T4)** per ANTICHEAT.md (~4 days). It's the hook; testers are recruited on it.
2. **PvP raids (T7)**: raid other players' saved bases, 10% loot, shields 8 h, revenge, defense log, trophies/leagues (Yeoman→Sovereign), matchmaking by trophies; computer bases kept only for tutorial + backup.
3. **Clash completeness pass**: walls placement + wall upgrades, 3rd defense (Trebuchet, splash), 4th troop (Ram, targets defenses/walls), obstacles to clear, builder count 2 (+3rd earned via streak?), spells? (out), clan (out).
4. **Art pass**: Kenney CC0 medieval/castle kits as base meshes for all buildings at 3 visual tiers (lv1-3, 4-6, 7-8); troops from a free CC0 character kit (Kenney "Mini Characters"/Quaternius) with simple animations; HUD icons and logo generated free in Canva/Figma; one consistent palette; UI frame. Sounds: free CC0 SFX (Kenney audio).
5. **Tutorial + phone layout (T5)**.
6. **Private link + tester kit + survey (T6)**, verifier on the deployed link, then owner's final go.

## Open risks
- Engine limits for animation/skinned meshes and asset size on the web.
- Art consistency when mixing kits.
- Too few testers online for PvP to feel alive.
- Balance with PvP loot vs study boosts.
