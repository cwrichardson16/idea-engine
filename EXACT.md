# Exploring Exact by Expo

Hands-on notes from cloning Exact ([github.com/expo/exact](https://github.com/expo/exact)), running its setup,
and building its "Grow a Garden" sample game. November 26, 2026.

## What it is

- An Expo project, **MIT license** (free).
- One codebase builds **real native apps** (iOS, Android, macOS, Linux; Windows early) **and the web**.
- Built to be driven by coding agents: build, run, screenshot, tap and verify from scripts.
- Has an optional **3D game engine**: gameplay in **Rust**, menus/HUD in Exact's UI language (**Contract**).
- Packages for in-app purchases, Apple/Google Pay, push notifications, a Convex realtime backend, and over-the-air updates.
- **Pre-1.0 and "changes daily"**: no packaged release, no API stability promise.

## What I actually did

| Step | Result |
|---|---|
| Clone + `exact setup` on Linux | **Worked** for web and Linux after one workaround (rustup self-update had to be turned off). iOS builds need a Mac (or extra Apple SDK setup). |
| Build the garden sample for web | **Worked after a fix:** the sample's dependency lock was out of date with Exact itself (`shells.mjs --lock` fixed it). That's the "changes daily" risk in practice. First build ~5–6 min; download is ~3.5 MB gzipped. |
| Run it in a browser | **Title screen works.** The simulation runs. |
| See the 3D world | **Not possible in the cloud container:** it has no graphics card, and Exact's web renderer **requires WebGPU with no fallback** (no WebGL). Exact's own screenshot proof failed the same way here. Needs a real computer. |
| Anki's built-in browser (Anki 26.09, Qt 6.12, Chromium 140) | **Has the WebGPU API**, so an Exact game *could* run inside Anki. Unconfirmed that it draws on a real GPU; needs a test on a real computer. |

## The sample game

"Grow a Garden": walk a gardener, plant, water, harvest, sell, fill market orders, expand.
"It keeps growing while you are away" (offline timers). That's very close to our base-builder's economy loop.
Code size is small: ~5,600 lines (Rust + UI) for the whole game.

## Engine features that matter for us

| Need | Exact |
|---|---|
| 3D buildings and troops | 3D meshes, imported `.glb`/`.gltf` models (Blender, Kenney kits), skeletal animation, sprites |
| Timers that run offline | Shown in the garden sample |
| Shop / economy | Built-in shop and spawner helpers |
| Troops walking to targets | Built-in steering (seek, arrive, flee, school) |
| Battle replays + anti-cheat | **Deterministic simulation:** same inputs give the same result, checked by hash. The server can re-run a raid in Rust to verify it. |
| Save games | Built-in saves |
| Purchases | `@exact/purchases` (App Store / Google Play), `@exact/payments` |
| Multiplayer | No built-in networking; we'd use a backend (Convex package exists, or Supabase) for async raids/clans, which is all our design needs |

## Verdict

**Good fit for a 3D hobby project**, with these conditions:
- **Develop on a real computer with a GPU** (ideally a Mac, since Exact's quick start targets Apple Silicon). The cloud container can write and test game logic, but can't show the 3D.
- **Expect breakage when updating Exact.** Pin to one commit and update deliberately.
- **Gameplay is Rust.** Fine with an AI writing most of it; harder to hand-edit.
- **Windows is early.** A web build covers Windows users in Chrome/Edge.

Fallback if Exact doesn't work out: a web 3D engine such as Three.js/Babylon.js (WebGL, runs everywhere), or Godot.
