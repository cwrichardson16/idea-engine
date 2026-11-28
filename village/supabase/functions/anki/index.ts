// Grand Rounds: the Anki link.
//  {action:"make"} with the player's bearer token → an 8-letter pairing code (a new one revokes the old).
//  {action:"pair", code, baseline:{day:count}} from the add-on → ok (stores the 30-day baseline once).
//  {action:"reviews", code, day, count, last_id} from the add-on → credits the day's total
//    (count = all qualifying reviews that review day so far; repeats are harmless,
//    an old last_id is refused, the server caps the day at 1500 and 1500 per rolling 24 h).
// Wrong codes: 10 tries per hour per address.
import { createClient } from "npm:@supabase/supabase-js@2";

const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, apikey, content-type, x-client-info",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};
const LETTERS = "ABCDEFGHJKMNPQRSTUVWXYZ23456789";
const WRONG_PER_HOUR = 10;

function json(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { ...CORS, "Content-Type": "application/json" } });
}
async function sha256(s: string) {
  const d = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(s));
  return Array.from(new Uint8Array(d)).map((b) => b.toString(16).padStart(2, "0")).join("");
}
function pick(n: number) {
  const r = crypto.getRandomValues(new Uint8Array(n));
  return Array.from(r).map((b) => LETTERS[b % LETTERS.length]).join("");
}
const today = () => Math.floor(Date.now() / 86_400_000);

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response("ok", { headers: CORS });
  if (req.method !== "POST") return json({ error: "POST only" }, 405);
  const admin = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!, {
    auth: { persistSession: false },
  });
  // deno-lint-ignore no-explicit-any
  let body: any = {};
  try { body = await req.json(); } catch { /* empty */ }

  if (body.action === "make") {
    const token = (req.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
    const { data: who, error } = await admin.auth.getUser(token);
    if (error || !who.user) return json({ error: "sign in first" }, 401);
    const code = pick(8);
    const salt = pick(16);
    const hash = await sha256(salt + code);
    const { data: old } = await admin.from("pairings").select("player_id").eq("player_id", who.user.id).maybeSingle();
    const { error: e2 } = old
      ? await admin.from("pairings").update({ salt, code_hash: hash, revoked: false }).eq("player_id", who.user.id)
      : await admin.from("pairings").insert({ player_id: who.user.id, salt, code_hash: hash });
    if (e2) return json({ error: "could not make a code" }, 500);
    return json({ code });
  }

  if (body.action !== "pair" && body.action !== "reviews") return json({ error: "unknown action" }, 400);

  const ip = "anki:" + (req.headers.get("x-forwarded-for") ?? "unknown").split(",")[0].trim();
  const since = new Date(Date.now() - 3_600_000).toISOString();
  const { count: wrong } = await admin.from("restore_attempts").select("id", { count: "exact", head: true })
    .eq("who", ip).gte("at", since);
  if ((wrong ?? 0) >= WRONG_PER_HOUR) return json({ error: "too many wrong codes, wait an hour" }, 429);
  const code = String(body.code ?? "").toUpperCase().replace(/[^A-Z0-9]/g, "");
  const { data: rows } = await admin.from("pairings").select("player_id, salt, code_hash, revoked, last_review_id, baseline")
    .eq("revoked", false);
  let p: { player_id: string; last_review_id: number; baseline: unknown } | null = null;
  if (code.length === 8) {
    for (const r of rows ?? []) {
      if ((await sha256(r.salt + code)) === r.code_hash) { p = r; break; }
    }
  }
  if (!p) {
    await admin.from("restore_attempts").insert({ who: ip });
    return json({ error: "that code isn't linked to a village (get a new one in the game)" }, 404);
  }

  if (body.action === "pair") {
    if (p.baseline == null && body.baseline && typeof body.baseline === "object") {
      const clean: Record<string, number> = {};
      for (const [d, n] of Object.entries(body.baseline).slice(0, 31)) {
        const day = Number(d), c = Number(n);
        if (Number.isInteger(day) && Number.isFinite(c) && c >= 0) clean[String(day)] = Math.min(Math.floor(c), 5000);
      }
      await admin.from("pairings").update({ baseline: clean }).eq("player_id", p.player_id);
    }
    return json({ ok: true, last_id: p.last_review_id });
  }

  // reviews
  const day = Number(body.day), n = Number(body.count), lastId = Number(body.last_id);
  if (!Number.isInteger(day) || Math.abs(day - today()) > 2) return json({ error: "that day is out of range" }, 400);
  if (!Number.isInteger(n) || n < 0 || !Number.isInteger(lastId) || lastId <= 0) return json({ error: "bad numbers" }, 400);
  if (lastId <= p.last_review_id) return json({ ok: false, duplicate: true, last_id: p.last_review_id });
  const { data: got, error: e3 } = await admin.rpc("credit_day", { p_player: p.player_id, p_day: day, p_anki: n, p_honour: null });
  if (e3) return json({ error: "could not count" }, 500);
  await admin.from("pairings").update({ last_review_id: lastId }).eq("player_id", p.player_id);
  return json({ ok: true, counted: got, last_id: lastId });
});
