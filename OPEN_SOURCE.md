# Open source we can use or learn from

Researched November 2026. **Check each license before copying any code or art.**

## Games like ours

**No good open-source game in this genre to fork.** The search turned up only small student or hobby projects
(e.g. a Qt/C++ tower-defense semester project, an early-stage sandbox called Rise of Realms). Nothing worth building on,
so we build our own, using the libraries below.

## Game engine & libraries

| Project | What it gives us | License |
|---|---|---|
| [Phaser 4](https://phaser.io) | 2D web game engine: sprites, input, camera, tilemaps (top-down *and* isometric built in). v4 stable since April 2026. | MIT |
| [Phaser isometric tilemap examples](https://editor.phaser.io/examples-show/806) | Official examples, if we go isometric later | MIT |
| [isoisoiso](https://github.com/briwa/isoisoiso) | Small TypeScript + Phaser isometric demo with pathfinding | MIT |
| Pathfinding libraries (pathfinding.js, EasyStar.js) | Troops finding their way around buildings and walls | MIT |

## Art: the look

The genre's look is bright, chunky, cartoony and readable at small sizes, usually at an angle (isometric).
Free art in that exact style is rare. Most free packs are pixel art or simple shapes. Mixing packs from different artists looks cheap,
so **pick one pack for the prototype, and plan on one consistent style (likely commissioned) before public launch.**

| Option | Look | Cost / license | Fit |
|---|---|---|---|
| **Tiny Swords** by Pixel Frog ([itch.io](https://pixelfrog-assets.itch.io/tiny-swords)) | Charming medieval pixel art, top-down: castles, towers, houses, knights, archers, goblins | Name-your-price. **Verified:** personal and commercial use, edits allowed, credit optional; no redistributing or reselling the art. Older version is CC0. Optional $15 enemy pack. | **Best free prototype look** |
| **Kenney: Medieval RTS** ([kenney.nl](https://kenney.nl/assets/medieval-rts)) | Clean, simple, top-down | Free, CC0 (no credit needed) | Safe fallback; plainer |
| **Kenney 3D kits** (Castle Kit, Retro Medieval Kit, Fantasy Town Kit) rendered to 2D at an angle | Consistent isometric look, any angle we want | Free, CC0 | Cheap route to the isometric look; needs a Blender render step |
| Paid isometric kits (e.g. [Isometric Tower Game Kit](https://mobilegamegraphics.itch.io/isometric-tower-game-kit), ~$10; similar packs on CraftPix) | Cartoon isometric, closest to the genre | Paid; check each license | Good look, but packs rarely match each other |
| [OpenGameArt: Isometric Enemy Base](https://opengameart.org/content/isometric-enemy-base) | Isometric pixel art, 117 assets | Page says commercial use allowed | Small; pixel style |
| **Commissioned artist** | Our own distinctive style | Usually a few thousand dollars and up for a core set | **Before public launch** |
| AI-generated | Anything | Cheap | Hard to keep consistent; some players dislike it |

**Open-source repo caution:** licensed art (like Tiny Swords) must stay out of the public add-on repo. See PLATFORMS.md.

**Avoid:** packs licensed "NoDerivatives" (e.g. BuzPin Studio's isometric packs). We can't edit or recolor them.

## Anki add-ons to learn from

These show how to hook reviews, show a game window inside Anki and talk to a server. **Read for patterns only.** Several use
Pokémon characters, which we don't touch.

| Add-on | Why it's useful |
|---|---|
| [Ankimon](https://github.com/IdreesInc/ankimon) | Battles inside Anki, driven by reviews. Closest to what we're building. PyQt6 (current Anki). |
| [Pokemanki](https://github.com/sivenchinniah/Pokemanki) | The original game-like add-on. **Lesson:** it broke on Anki 24.11 because it relied on old PyQt5; target current Anki. |
| Anki Leaderboard ([forum thread](https://forums.ankiweb.net/t/add-on-support-thread-anki-leaderboard-by-shige/51634)) | Social add-on with its own server; 3,000+ active users. Proof that social features work in an add-on, and a model for hosting. Has had maintainer handovers. |
| [AnkiHabitica](https://github.com/eshapard/AnkiHabitica) | Sends reviews to an outside game (Habitica). The same "reviews → server → rewards" pattern we need later. |
| [Anki add-on docs](https://addon-docs.ankiweb.net/) | Official guide: hooks, web views, config |
