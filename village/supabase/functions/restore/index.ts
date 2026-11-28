// Grand Rounds: move a player's account to another device.
//  {action:"make"} with the player's bearer token → a one-time 8-letter code.
//  {action:"claim", code} from the new device → that account's sign-in
//  (a fresh password, so the old device is signed out). 5 tries per hour per address.
import { createClient } from "npm:@supabase/supabase-js@2";

const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, apikey, content-type, x-client-info",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};
const LETTERS = "ABCDEFGHJKMNPQRSTUVWXYZ23456789"; // no 0/O, 1/I/L
const TRIES_PER_HOUR = 5;

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

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response("ok", { headers: CORS });
  if (req.method !== "POST") return json({ error: "POST only" }, 405);
  const admin = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!, {
    auth: { persistSession: false },
  });
  let body: { action?: string; code?: string } = {};
  try { body = await req.json(); } catch { /* empty */ }

  if (body.action === "make") {
    const token = (req.headers.get("authorization") ?? "").replace(/^Bearer\s+/i, "");
    const { data: who, error } = await admin.auth.getUser(token);
    if (error || !who.user) return json({ error: "sign in first" }, 401);
    const code = pick(8);
    const salt = pick(16);
    const { error: e2 } = await admin.from("restore_codes").upsert({
      player_id: who.user.id, salt, code_hash: await sha256(salt + code), created_at: new Date().toISOString(),
    });
    if (e2) return json({ error: "could not make a code" }, 500);
    return json({ code });
  }

  if (body.action === "claim") {
    const who = (req.headers.get("x-forwarded-for") ?? "unknown").split(",")[0].trim();
    const since = new Date(Date.now() - 3_600_000).toISOString();
    const { count } = await admin.from("restore_attempts").select("id", { count: "exact", head: true })
      .eq("who", who).gte("at", since);
    if ((count ?? 0) >= TRIES_PER_HOUR) return json({ error: "too many tries, wait an hour" }, 429);
    await admin.from("restore_attempts").insert({ who });
    const code = String(body.code ?? "").toUpperCase().replace(/[^A-Z0-9]/g, "");
    if (code.length !== 8) return json({ error: "that code doesn't look right" }, 404);
    const { data: rows } = await admin.from("restore_codes").select("player_id, salt, code_hash");
    let found: string | null = null;
    for (const r of rows ?? []) {
      if ((await sha256(r.salt + code)) === r.code_hash) { found = r.player_id; break; }
    }
    if (!found) return json({ error: "no account has that code" }, 404);
    const { data: u } = await admin.auth.admin.getUserById(found);
    if (!u.user?.email) return json({ error: "no account has that code" }, 404);
    // Saves signed in before this moment are refused (put_save checks moved_at).
    const { error: e4 } = await admin.from("players").update({ moved_at: new Date().toISOString() }).eq("id", found);
    if (e4) return json({ error: "could not restore" }, 500);
    const raw = crypto.getRandomValues(new Uint8Array(24));
    const password = btoa(String.fromCharCode(...raw));
    const { error: e3 } = await admin.auth.admin.updateUserById(found, { password });
    if (e3) return json({ error: "could not restore" }, 500);
    await admin.from("restore_codes").delete().eq("player_id", found);
    return json({ email: u.user.email, password });
  }

  return json({ error: "unknown action" }, 400);
});
