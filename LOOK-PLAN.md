# Look pass 2: "more Clash-like cartoon" (Conner, 2026-11-27)

Goal: the village reads like a cartoon base-builder in the Clash family (not a copy): steep diamond view, base fills the screen, bright saturated colours, ink outlines, chunky HUD. Must not break picking, drag-to-pan, tap-to-select, placing, saves or tests.

## Already done (uncommitted, building now)
- `tools/add_outline.py` adds engine toon outlines (`_OUTLINE_NORMAL/_OUTLINE_REACH` + `extras.outline`) and rim light (`material.extras.toon.rim`) to the 10 Kenney GLBs in `village/art/`. Bake/validation accepted them: `bun exact.mjs test-rust` 36 pass (tests load art via `sim_with_art!`).
- `logic/src/lib.rs` setup: brighter sky `#6ec6f5`, ambient 0.75, linear fog 60–110 m (was dense height fog), grass `#4fae2e`, a dirt rim plane under the grid, grid `#7fcf45`, warm sun 12000 lux.

## Next steps, in order
1. **Diamond camera.** `EYE - LOOK` becomes a diagonal offset, about (13, 26, 13): pitch ~55°, yaw 45°, so the square grid shows as a diamond. Default zoom so the 30×30 grid fills a 16:9 view. `tile_at` already derives from `camera_pose`, so picking follows.
2. **Drag in screen space.** `steer` maps drag x to the camera's screen-right ground vector (1,0,-1)/√2 and drag y to the toward-camera ground vector (1,0,1)/√2, so the ground follows the finger. Pan clamp stays a ±PAN_LIMIT square in world x/z.
3. **Grid only while placing.** Name the grid plane; each draw sets its material to plain grass when `ui.placing` is empty, grid lines when placing.
4. **Chunky HUD** in `app.contract`: bold font, dark text outline/shadow, thicker rounded borders on pills and buttons (only properties Contract supports; `contract build` must stay `[]`).
5. **Later (after battles, Phase B):** swap Kenney for the KayKit medieval pack (CC0), run `fix_glb_uri.py` + `add_outline.py` on it.

## Checks
- `bun exact.mjs test-rust` passes; existing picking tests updated to the new camera, plus new tests:
  - with the diamond camera, the screen centre picks a tile near the grid centre, and all four grid corners are on screen at default zoom (fills, nothing cut off);
  - a drag keeps the tile under the finger roughly under the finger (within 1 tile) after `steer`.
- `contract build` gives `[]`.
- Chrome: screenshot before/after; tap selects the Town Hall; drag right moves the village right; placing shows grid, cancelling hides it.

## Things I'm unsure about
- Whether the outline width (0.035 m, min 1.4 px) reads well at the new zoom.
- Whether a steeper camera makes the 2.7× Town Hall tower look stubby.
