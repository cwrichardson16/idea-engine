// Grand Rounds: create a private player account (no email needed).
// The project keeps anonymous sign-ins off, so this function makes a real
// user with a random address on a reserved .invalid domain and a random
// password, and hands both back once. The game keeps them on the device and
// signs in with them. A global cap per hour stops anyone from mass-creating accounts.
import { createClient } from "npm:@supabase/supabase-js@2";

const CORS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Headers": "authorization, apikey, content-type, x-client-info",
  "Access-Control-Allow-Methods": "POST, OPTIONS",
};
const JOINS_PER_HOUR = 60;

function json(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { ...CORS, "Content-Type": "application/json" } });
}

Deno.serve(async (req) => {
  if (req.method === "OPTIONS") return new Response("ok", { headers: CORS });
  if (req.method !== "POST") return json({ error: "POST only" }, 405);
  const admin = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!, {
    auth: { persistSession: false },
  });
  const since = new Date(Date.now() - 3_600_000).toISOString();
  const { count, error: countError } = await admin
    .from("players").select("id", { count: "exact", head: true }).gte("created_at", since);
  if (countError) return json({ error: "try again" }, 503);
  if ((count ?? 0) >= JOINS_PER_HOUR) return json({ error: "too many new players right now, try again soon" }, 429);
  const email = `${crypto.randomUUID()}@players.grandrounds.invalid`;
  const raw = crypto.getRandomValues(new Uint8Array(24));
  const password = btoa(String.fromCharCode(...raw));
  const { data, error } = await admin.auth.admin.createUser({ email, password, email_confirm: true });
  if (error || !data.user) return json({ error: "could not create a player" }, 500);
  const { error: insertError } = await admin.from("players").insert({ id: data.user.id });
  if (insertError) return json({ error: "could not create a player" }, 500);
  return json({ email, password });
});
