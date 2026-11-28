session: 2450836a-499e-482d-919d-4a3f50256f74
Mid-flight: SPEC-3 build. T0 done; T1 done (test web ok); T2 done in code + fix for skeptic BLOCK (power-hour save wipe); skeptic round 2 pending on the fix commit.
Next exact actions:
1. If skeptic round 2 SHIP: start dev server (`bun exact.mjs web --port 8765` from village/, background), one Chrome look at T1+T2 (names, hidden dev, 5 taps on Hospital badge, study panel after 50 reviews → chest toast, power ⚡), screenshot paths into STATE.
2. T3: create ONE free Supabase project (Conner approved; $0 only; check list_organizations/cost first, refuse anything paid), tables + RLS + put_save RPC per SPEC-3 T3, then data crate per NET-RECIPE.md, spike one round trip from web build.
Conner: keep building all of SPEC-3; do NOT share the link with testers until his final go.
