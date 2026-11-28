# Network recipe for T3 (from the tennis sample; helper research 2026-11-28, paths in ~/Projects/exact)
- app.json: {"game":{"assets":true,"data":{"crate":"village-data","type":"Supa"}}} (tennis app.json:4; game/README.md:1031)
- village/data/Cargo.toml: [package] name="village-data", edition="2021", workspace="../.shells"; [dependencies] exact-runner={workspace=true}, serde_json="1" (tennis/data/Cargo.toml). No `lock` needed while deps are in the SDK lock.
- data/src/lib.rs: use exact_runner::{Answer, DataError, DataSource, Outcome, Request, Store, Value}; #[derive(Default)] pub struct Supa; impl DataSource:
  - grants() -> "net.fetch https://<ref>.supabase.co" (one origin per line; secret.keep <name> for tokens via store.set/get)
  - query(source,args): bake placeholder → Ok(Value::str("")) for our source, Err(DataError::UnknownSource) otherwise
  - answer(store,source,args): input = args[0].as_str; empty → Answer::Now(Value::str("")); else Answer::Later(Request::post_json(url, body).header("apikey",ANON).header("authorization",format!("Bearer {jwt}"))) ; Request::get(url); PATCH via Request{method:"PATCH".into(),..}
  - parse(store,source,args,outcome): Outcome::Response(r) → r.status/r.body; Outcome::Failed{kind,..} (Network/Refused/Timeout/Aborted/...) → return {"id":..,"ok":false,"error":..} as data
  (tennis/data/src/lib.rs:12-202; runner/src/request.rs:404,452-466,567,1095-1190)
- Contract: `resource reply = supa(hud.ask) as shape string` (world puts a numbered JSON request in hud.ask). Answer back to world as a live Options field: `#[live] pub net: String` on Options + `net=reply` on the canvas surface; world reads it each tick and applies only matching id (tennis logic lib.rs:26-29,140; app.contract:21-30,56).
- Gotchas: web runs the crate as wasm, CORS failure = Network; grants checked by origin; never ship service_role key; key replies on request id; tests: `answer fetch "<prefix>" with 200 {...}` / `fail fetch "<prefix>"` (docs/agent-pitfalls.md:1626-1644). Adding game.data makes native Apple builds use their own host (slower first build).
