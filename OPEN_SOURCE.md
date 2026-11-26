# Open source we can use or learn from

Researched November 2026. **Check each license before copying any code or art.**

## Games like ours

**No good open-source game in this genre to fork.** The search turned up only small student or hobby projects
(e.g. a Qt/C++ tower-defense semester project, an early-stage sandbox called Rise of Realms). Nothing worth building on,
so we build our own, using the libraries below.

## Game engine & libraries

| Project | What it gives us | License |
|---|---|---|
| [Phaser](https://phaser.io) | 2D web game engine: sprites, input, camera, tilemaps (top-down *and* isometric built in) | MIT |
| [Phaser isometric tilemap examples](https://editor.phaser.io/examples-show/806) | Official examples, if we go isometric later | MIT |
| [isoisoiso](https://github.com/briwa/isoisoiso) | Small TypeScript + Phaser isometric demo with pathfinding | MIT |
| Pathfinding libraries (pathfinding.js, EasyStar.js) | Troops finding their way around buildings and walls | MIT |

## Art (for the prototype and maybe beyond)

| Pack | Notes | License |
|---|---|---|
| [Kenney: Medieval RTS](https://kenney.nl/assets/medieval-rts) | 2D top-down, ~120 assets: buildings, units, tiles. **Best fit for the v1 prototype.** | CC0 (free, no credit needed) |
| [Kenney: Retro Medieval Kit](https://kenney.nl/assets/retro-medieval-kit) | 3D, ~100 assets; could be rendered at an angle for an isometric look | CC0 |
| [Kenney: Castle Kit](https://kenney-assets.itch.io/castle-kit) | 3D castle pieces | Check page |
| [OpenGameArt: isometric medieval buildings](https://opengameart.org/content/isometric-medieval-buildings) | Isometric buildings | Check page |

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
