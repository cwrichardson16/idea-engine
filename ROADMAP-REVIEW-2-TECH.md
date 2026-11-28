# ROADMAP + GOAL-LOOP review, round 2: technical ("will it work and finish?")

Reviewed: ROADMAP.md, GOAL-LOOP.md, village/ code, engine docs at village/.exact/checkout (-> ~/Projects/exact @ 46f0aefe). Read-only, no builds or tests run.

## Blocking (plan text must change before the loop starts)

B1. A rules change can silently wipe a village, locally and in the cloud. CONFIRMED.
- `Town::decode` ends in `town.valid().then_some(town)` (economy.rs:969). `valid()` checks values that step 1 and later steps will retune: production `holds` (banked_ok, economy.rs:995-998), storage caps (economy.rs:1013-1014), `housing_cap` (1012), `max_count`, `size` (grid overlap, 985-993), `max_level` (982). It also requires `army.len() == TROOPS.len()` (economy.rs:913) and `stars.len() == CAMP_COUNT` (1008-1010). So adding the Ram (step 10) or changing the number of camps (step 6) refuses every older save.
- When decode refuses, `load` just returns and leaves the starter village in place (lib.rs:445-449). The world's next save differs, so `dirty` is set (cloud.rs:103-104) and uploaded against the current cloud version (cloud.rs:172-179). The server accepts it and the real village is gone. The same thing already happened once (STATE.md, T2 skeptic round 1: "decode refuse the save → fresh village overwrote it").
- Why it blocks: ROADMAP.md:25 plans to retune numbers during week 1 of the test, and the goal (GOAL-LOOP.md:4) says "nothing ... loses progress".
- Fix (add to step 1's done-check): (a) decode clamps rule-dependent values (excess gold or troops, banked amounts, extra camps) instead of refusing, and pads or truncates per-troop and per-camp vectors; (b) a refused save is never replaced: keep it, show an error, and do not upload; (c) a test with frozen save strings from every past format (v1, army, campaign, study, and each new one) that must still decode after every rules change.

B2. Phone testers: there is no WebGL fallback, and nothing in the plan tests a real phone. CONFIRMED.
- "No WebGL renderer exists". Without WebGPU, the page shows a refusal instead of the game (game/README.md:372-382).
- The finish line only tests Chrome at 390×844 (GOAL-LOOP.md:9). Nothing in the engine docs mentions Safari (grep of game/README.md, render/README.md, docs/testing.md: no hits). The engine docs report a Pixel 5 drawing the starter at 16 fps under the `High` quality setting (game/README.md:366).
- Whether iOS Safari's WebGPU runs this engine is UNVERIFIED.
- Fix: in step 2, the owner opens the spike build on his iPhone and on one Android phone (owner-only item, ~5 min) and reports fps and load time. Add "real iPhone + Android" to finish line 3. The tester guide lists the supported browsers and OS versions.

## Should-fix (CONFIRMED gaps; not blocking)

S1. PvP (step 6) needs server and client work the roadmap doesn't list, so its done-check should name it:
- RLS lets a player read only their own player and save rows (001_init.sql:52-54). Listing opponents and the top-10 needs SECURITY DEFINER RPCs.
- No `raids` table exists (only `players.shield_until`; grep of migrations).
- No trophies column. `players.level` is never updated (put_save, 001_init.sql:60-93).
- The data crate passes only `kind == "review"` events (data/src/lib.rs:106), so a "raided" loot-loss event would be dropped.
- The server can't compute "10% of storage" because the save is opaque client text. It must parse `split_part` of the text or trust the attacker's capped claim.
- Skip abandoned starter accounts left behind by a restore claim (STATE.md "T7 NOTE").
- With all of that, 3–4 days is tight.

S2. Events arrive only on sign-in or conflict (cloud.rs:240, 262; no other `Step::Fetch`). A player who is online never sees a raid (step 6) or new Anki reviews (step 4) until they reload. Add a periodic refetch (for example every 60 s while not raiding) to step 4.

S3. Step order: step 3 adds hit and death animations (ROADMAP.md:9), but the animated troops arrive in step 8 (ROADMAP.md:14), and the spike has one troop. Either do step 3 with HP bars and shots only and move the animations to step 8, or accept the rework.

S4. Keep battle truth in `battle.rs`. It is pure whole-number rules (battle.rs:1-3), which is what makes the balance sim (finish line 6) and any PvP check possible. The art review suggests firing damage on clip markers (ROADMAP-REVIEW-ART.md:63). That would tie results to loaded models, and animated models must be in blocking `ASSETS` (game/README.md:1133). Markers should be cosmetic only.

S5. Finish line 6 can be passed with a weak sim. It has no player policy and no numbers (GOAL-LOOP.md:12). Fix both up front:
- a scripted builder (always upgrade the cheapest eligible building, collect every N hours, with and without 100 reviews a day);
- a scripted attacker (a fixed drop pattern);
- bounds such as Keep level ranges at day 1, 3 and 7, and an average of 2 stars or fewer against an equal-level base.

S6. The roadmap needs `building_hp(kind)` to take a level (battle.rs:85-93, no level today). This confirms that step 1 must come before step 6, which the order already does.

S7. Owner-only items missing from the loop's stop list:
- Anki is not installed on this Mac: no /Applications entry, no ~/Library/Application Support/Anki2. Finish line 4 therefore needs the owner, or an install, which is a new app. Ask once, up front.
- Blender is not installed either. The art pipeline must use glTF-Transform via node (node and npx are present).

S8. Finish line 3's "< 10 s" needs a definition: the `web-build --release` artifact (production trust, scripts/help.mjs:67; STATE notes the spike was development trust), Chrome network throttling at "Fast 4G", and a cold cache. Add the art review's size test (ASSETS ≤ 2 MB gzip, ROADMAP-REVIEW-ART.md:78) to step 2.

## Checked and fine
- Move buildings (step 5) needs no save-format change: `encode` already stores x and z per building (economy.rs:840-842), and `valid` re-checks the grid.
- The engine supports skinned animation: Animator, layers, markers and sockets (engine/README.md:271-294). Limits are 256 joints and 256 nodes per animated model (engine/src/asset.rs:615-621), and KayKit's Knight has 41 joints (art review). Scale and shear on posable nodes are refused (asset.rs:611), so the spike must bake a real KayKit GLB.
- Size: about 3.6 MB per unstripped Adventurers GLB (ROADMAP-REVIEW-ART.md:40). Troops that animate must be in blocking ASSETS (game/README.md:1133), so stripping clips is required, not optional. Step 2 covers this.
- Today's web build is app.wasm 1.5 MB gzip plus gpu 0.8 MB gzip, 13 MB uncompressed dist (STATE.md T0).
- The balance sim is feasible. Battles are pure and deterministic (battle.rs:1-3), and a builder-days affordability test already exists (tests/battle.rs:197-198).
- Loop rules: one writer, a 3-round review cap, heavy jobs one at a time, the 2-failed-fixes stop, commit and push to the branch, and STATE evidence pointers are all sound. The private-link deploy and the free Supabase project are approved (STATE.md "Conner approvals"). The UNVERIFIED part is whether "a tuning table we can change without a release" (ROADMAP.md:25) has a mechanism. Exact `TABLES` are baked assets (game/README.md:1163-1170), so changing one on the static host means a redeploy, and it must also satisfy B1.

VERDICT: BLOCK (B1 and B2 are short text additions to step 1, step 2 and finish line 3; after that, SHIP)

---

# Round 3 (re-review of ROADMAP v2 + GOAL-LOOP v2)

## Round-2 findings, now addressed: all CONFIRMED in the text
- B1 save safety: ROADMAP.md:33-37 adds the three tests: frozen saves from every format, clamp instead of refuse, a failed load never uploads. Lines 37 and 118 add the SAVE_CONTRACT rule (bump + migration + frozen string), including during the test week. GOAL-LOOP finish line 7 adds the tests.
- B2 real phones: step 2 has an iPhone + Android gate (ROADMAP.md:51-52) and a decision if the iPhone fails. Finish line 3 has the real-phone runs (GOAL-LOOP.md:11).
- S1 PvP server work: the pieces are listed (ROADMAP.md:75-79). Raid events must pass the data crate, which today drops them (data/src/lib.rs:106).
- S2 periodic re-fetch: every 60 s, with a 90 s no-reload proof (ROADMAP.md:66, 79). It fits the code: the step loop sends only when `pending == 0` (cloud.rs:126), so a refetch can't drop an in-flight save reply. After the first sync, a fetch takes the cloud save only when its version is higher (cloud.rs:376).
- S3/S4: animations move to step 8. Step 3's fixed-seed test locks damage timing to battle.rs (ROADMAP.md:55, 86).
- S5 balance sim: a fixed builder and attacker strategy with numeric pass bars (ROADMAP.md:39-43).
- S7 owner items: listed up front (GOAL-LOOP.md:29).
- S8 load time: release build, cold cache, Fast 4G, 4× CPU, measured to a tappable Keep (ROADMAP.md:49).

## New, non-blocking
- N1. The phone pass bar asks for "fps (dev overlay)" (ROADMAP.md:51). I found no fps overlay in the engine docs (grep "fps|overlay" in game/README.md hits only max_fps and particle fps), and production-trust builds omit dev markers (game/README.md:669). UNVERIFIED that one exists. Fix: show fps in a debug chip the game draws itself, or use pass = "smooth to the eye + seconds to the village".
- N2. Phase P plans a `pg_dump` backup (ROADMAP.md:117). pg_dump is not installed (`which pg_dump`: not found). The supabase CLI is present but needs the database password. Fix: make it a JSON export of the playtest tables through SQL, or install pg_dump on day 0.
- N3. Step 6's test checks that loot is applied once but not saturation. If the defender spent gold before the raid event arrives, the loss must stop at 0 (gold is u32). Add a test where the event is larger than the current gold. UNVERIFIED, because the code isn't written yet.
- N4. The step 6 SQL check says opponents get the "layout, never A's save text" (ROADMAP.md:77). The RPC therefore has to return only the building section (split on ';' or '#' in SQL), which is work beyond a plain select. It is feasible, and the 4–5 days probably absorb it.

## Proxy check
Each done-check ends at a source: a test with stated values, SQL row counts, a Chrome screenshot of a stated state, or a phone result from Conner. Weakening a check requires telling Conner (ROADMAP.md:28; GOAL-LOOP.md:7, 24). I found no check that a status name or green test alone can satisfy.

VERDICT (round 3): SHIP
