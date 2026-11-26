# SPEC: Slice 1, the village (Clash of C____)

Status: approved by default on 2026-11-26. Conner said "go for it build". The choices below are mine and can be changed.

## Goal
Get the first playable piece of the game running in 3D, in Chrome on the Mac, built with Exact by Expo. You see a village on a grid. You place buildings with gold or elixir. Mines and collectors fill up over time, and you tap them to collect into your storages. Builders run construction and upgrade timers, two at a time.

## Finish line (all must hold, with evidence)
1. The game's logic tests pass (`cargo test` in the game's logic crate), with the output shown.
2. Exact's own checks for the game pass: the build, and `prove` if the game uses it.
3. The game runs at http://127.0.0.1:<port> in Chrome. Screenshots show each feature below working.
4. A fresh `skeptic` review finds no unfixed correctness issues.
5. The work is committed and pushed to the branch `claude/eloquent-goodall-rrq9xv`. Not to main.

## Choices made (Conner can override)
- **Placeholder art.** Simple colored 3D blocks. Real models come later (Kenney, CC0).
- **Angled top-down camera** (decision V4).
- **Six building types** in this slice: Town Hall, Gold Mine, Elixir Collector, Gold Storage, Elixir Storage and Builder Hut. Barracks, troops, defenses and battles are slice 2.
- **Game location.** The game lives in this repo under `game/`, if Exact can run games from outside its own folder. Otherwise it lives in `~/Projects/exact/game/games/clash/` and is mirrored here. This is recorded in STATE.md.
- **Gameplay numbers.** Every cost, rate and timer is in one table in the code, with short timers for testing (seconds, not hours).
- **Offline progress.** Mines keep producing and timers keep running while the game is closed, using real clock time.

## Feature checklist
Each feature names how it's checked and what the proof is.

- [ ] **F1. Village grid and camera.**
  - What it is: a 3D grid of about 30×30 tiles. The Town Hall starts in the middle, and the starting gold and elixir show in the top bar.
  - Done check: the page renders the grid, the Town Hall and the counters.
  - Proof: a screenshot.
- [ ] **F2. Shop and placement.**
  - What it is: the shop lists buildings with their costs. You pick one, then tap a free tile to place it. The cost is taken, and you can't overlap other buildings or go off the grid. The Town Hall level caps how many of each building you can own.
  - Done check: logic tests for cost, overlap, bounds and caps; placing a building in Chrome.
  - Proof: test output and a before/after screenshot.
- [ ] **F3. Production and collecting.**
  - What it is: mines and collectors fill at their rate up to their own cap. Tapping one moves the resource into storage, up to the storage limit, which comes from the Town Hall plus the storage buildings. Anything over the limit stays in the mine.
  - Done check: logic tests for rate, mine cap and storage overflow; collecting in Chrome.
  - Proof: test output and a screenshot.
- [ ] **F4. Builders and timers.**
  - What it is: placing or upgrading a building uses a free builder until its timer ends. There are two builders: one from the start, plus a Builder Hut. With no free builder you can't build. Each building has 3 levels with rising cost, time and output, and the Town Hall level caps building levels.
  - Done check: logic tests for builder count, the timer finishing and level effects; an upgrade with a timer counting down in Chrome.
  - Proof: test output and a screenshot.
- [ ] **F5. Save and offline progress.**
  - What it is: the village survives a page reload. Time away is counted: mines fill and timers finish.
  - Done check: a logic test for "advance N seconds offline"; reloading in Chrome shows the same village with progress made.
  - Proof: test output and before/after-reload screenshots.
- [ ] **F6. Speed-ups (study hook stub).**
  - What it is: a dev-only "Do a review" button gives Knowledge. Knowledge or gems can finish a timer instantly, at a cost based on time left. Players start with a few gems.
  - Done check: logic tests for the skip cost and that Knowledge only comes from reviews; skipping in Chrome.
  - Proof: test output and a screenshot.

## What the player must NOT see
- A negative or impossible resource number.
- A building placed on top of another, or off the grid.
- A third builder working with only two.
- A timer that goes backwards or stalls after a reload.
- An error message, or a blank 3D screen.

## Out of scope for this slice
Troops, battles, defenses, walls, Anki, accounts/server, real payments, final art and the name.

## Premortem: this fails if…
1. **Exact can't run a game from outside its repo.** Check: confirm the dev server runs it from `game/`. If not, use the fallback above.
2. **Time math drifts or double-counts across save/reload.** Check: a logic test that advances time in pieces vs all at once and gets the same result; a reload test.
3. **Tapping a 3D tile hits the wrong tile.** Check: in Chrome, click a corner tile and the center tile, and screenshot that the building lands where clicked.
