# Handoff — 2026-11-26 (Mac, M1, macOS 15.7.4)

## Where we are
- Design docs are done (README, DESIGN, GAMEPLAY, CLASH_REFERENCE, V1, PLATFORMS, EXACT, OPEN_SOURCE).
- Mac setup done: Rust (rustup, ~/.cargo/bin) and Bun 1.4.5 (~/.bun/bin) installed. Exact cloned to ~/Projects/exact; `bun scripts/exact.mjs setup` says web is ready.
- Still missing: full Xcode (only needed for native macOS/iOS builds; the user installs it from the App Store).
- The garden sample's lock was refreshed with `bun game/app/shells.mjs game/games/garden --lock` (same fix as in the cloud).

## Mid-flight (resolved)
- Garden built and 3D confirmed in Chrome (see EXACT.md "Mac check"). Next: start the game project.

## Exact next action
1. When http://127.0.0.1:8765 answers, open it in Chrome and screenshot it to prove 3D renders on the M1's GPU (WebGPU).
2. Then start the game: `bun game/new.mjs` creates a game project in this repo. The first Rust slice is grid, building placement, gold/elixir trickle, and builders with upgrade timers.
