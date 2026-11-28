//! Cloud save (SPEC-3 T3). One request at a time goes out through `hud.ask`
//! (see village/data); its answer comes back in the live `net` argument.
//! Steps: join (once per device) → sign in → fetch the cloud save and any
//! review events → upload each changed save with the version it was based on.
//! A conflict means another device saved first: its save wins and loads here.
//! Offline: keep playing on the local save and retry.

use crate::economy::*;
use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Data)]
pub enum Step {
    /// Waiting for the page's credentials and the local save to load.
    #[default]
    Off,
    Join,
    SignIn,
    Fetch,
    Ready,
}

#[derive(Default, Resource)]
pub struct Cloud {
    /// The device's player account (from `join`, kept by the page).
    pub email: String,
    pub password: String,
    pub token: String,
    pub step: Step,
    /// The cloud save's version this device's save builds on.
    pub version: i64,
    pub next_id: u64,
    /// The request out now: its id and the ask text the HUD publishes.
    pub pending: u64,
    pub ask: String,
    /// The save text last accepted by the server, and the one in flight.
    pub sent: String,
    pub sending: String,
    /// The newest save text the page kept (`v2|ms|town@wall`), waiting to go up.
    pub dirty: String,
    /// Review events applied here and not yet marked applied on the server.
    pub applied: Vec<u64>,
    pub sending_events: Vec<u64>,
    /// Village ms before which nothing is sent (backoff, throttle).
    pub wait_until: u64,
    pub last_put: u64,
    /// The last line read from `net`, so one answer is handled once.
    pub seen: String,
    pub status: String,
    /// When the save this device loaded at Play was made (real ms); a cloud
    /// save made later came from another device and wins.
    pub local_wall: f64,
    /// After a conflict: the next fetch takes the cloud save whatever its time.
    pub take_cloud: bool,
    /// Has this device fetched once since Play? After that, only a higher
    /// cloud version (another device) replaces the village here.
    pub synced: bool,
}

const RETRY_MS: u64 = 15_000;
const SAVE_GAP_MS: u64 = 5_000;

impl Cloud {
    fn send(&mut self, kind: &str, token: &str, args: &[&str]) {
        self.next_id += 1;
        self.pending = self.next_id;
        let mut ask = format!("{}\t{kind}\t{token}", self.pending);
        for a in args {
            ask.push('\t');
            ask.push_str(a);
        }
        self.ask = ask;
    }
}

/// `cred <email> <password>` from the page: this device's account.
pub fn set_cred(w: &mut World, arg: &str) {
    let mut parts = arg.split_whitespace();
    if let (Some(e), Some(p)) = (parts.next(), parts.next()) {
        let mut c = w.resource_mut::<Cloud>();
        c.email = e.to_string();
        c.password = p.to_string();
    }
}

/// The page kept a new save: queue it for upload.
pub fn kept(w: &mut World, text: &str) {
    let mut c = w.resource_mut::<Cloud>();
    // The page re-saves every few seconds; only a changed village goes up.
    if c.dirty.is_empty() || town_of(&c.dirty) != town_of(text) {
        c.dirty = text.to_string();
    }
}

/// The village part of a kept save (`v2|ms|<town>@wall`).
fn town_of(text: &str) -> &str {
    let body = text.rsplit_once('@').map_or(text, |(b, _)| b);
    body.splitn(3, '|').nth(2).unwrap_or("")
}

/// One tick: read an answer, then send the next request if one is due.
pub fn step(w: &mut World, net: &str) {
    if !w.resource::<Ui>().loaded {
        return;
    }
    let now = now_ms(w);
    if !net.is_empty() && w.resource::<Cloud>().seen != net {
        w.resource_mut::<Cloud>().seen = net.to_string();
        answer(w, net, now);
    }
    let raid = fight::active(w);
    let mut c = w.resource_mut::<Cloud>();
    if c.pending != 0 || now < c.wait_until {
        return;
    }
    match c.step {
        Step::Off => {
            c.step = if c.email.is_empty() { Step::Join } else { Step::SignIn };
            c.status = "☁️ connecting".into();
        }
        Step::Join => c.send("join", "", &[]),
        Step::SignIn => {
            let (e, p) = (c.email.clone(), c.password.clone());
            c.send("signin", "", &[&e, &p]);
        }
        Step::Fetch => {
            // A cloud save is about to replace this one: never mid-raid.
            if c.take_cloud && raid {
                return;
            }
            let t = c.token.clone();
            c.send("state", &t, &[]);
        }
        Step::Ready => {
            let has_events = !c.applied.is_empty();
            if (c.dirty != c.sent || has_events) && !c.dirty.is_empty() && now >= c.last_put + SAVE_GAP_MS {
                let (t, base, text) = (c.token.clone(), c.version.to_string(), c.dirty.clone());
                let events: Vec<String> = c.applied.iter().map(|n| n.to_string()).collect();
                c.sending = text.clone();
                c.sending_events = c.applied.clone();
                c.last_put = now;
                let contract = SAVE_CONTRACT.to_string();
                c.send("save", &t, &[&base, &contract, &events.join(","), &text]);
            }
        }
    }
}

fn answer(w: &mut World, line: &str, now: u64) {
    let mut f = line.split('\t');
    let id: u64 = f.next().and_then(|n| n.parse().ok()).unwrap_or(0);
    let what = f.next().unwrap_or("");
    let rest: Vec<&str> = f.collect();
    let step = {
        let c = w.resource::<Cloud>();
        if id == 0 || id != c.pending {
            return;
        }
        c.step
    };
    {
        let mut c = w.resource_mut::<Cloud>();
        c.pending = 0;
        c.ask.clear();
    }
    match what {
        "offline" | "error" => {
            let mut c = w.resource_mut::<Cloud>();
            c.wait_until = now + RETRY_MS;
            c.status = if what == "offline" { "📴 offline: saved on this device".into() } else { "☁️ retrying".into() };
        }
        "auth" => {
            let mut c = w.resource_mut::<Cloud>();
            c.token.clear();
            // A device whose account is gone starts a new one.
            c.step = if step == Step::SignIn { Step::Join } else { Step::SignIn };
        }
        "ok" => match step {
            Step::Join => {
                if let (Some(e), Some(p)) = (rest.first(), rest.get(1)) {
                    let mut c = w.resource_mut::<Cloud>();
                    c.email = e.to_string();
                    c.password = p.to_string();
                    c.step = Step::SignIn;
                    drop(c);
                    w.emit(format!("cred {e} {p}"));
                }
            }
            Step::SignIn => {
                let mut c = w.resource_mut::<Cloud>();
                c.token = rest.first().unwrap_or(&"").to_string();
                c.step = Step::Fetch;
            }
            Step::Fetch => fetched(w, &rest),
            Step::Ready => {
                let mut c = w.resource_mut::<Cloud>();
                c.version = rest.first().and_then(|v| v.parse().ok()).unwrap_or(c.version);
                c.sent = std::mem::take(&mut c.sending);
                let done = std::mem::take(&mut c.sending_events);
                c.applied.retain(|e| !done.contains(e));
                c.status = "☁️ saved".into();
            }
            Step::Off => {}
        },
        "conflict" => {
            // Another device saved first: its save is the newest truth. Fetch
            // it again (with any events) and take it; events counted here but
            // not in that save are counted again from it (its last_event).
            let mut c = w.resource_mut::<Cloud>();
            c.sending.clear();
            c.sending_events.clear();
            c.applied.clear();
            c.take_cloud = true;
            c.step = Step::Fetch;
            c.status = "☁️ loading your newer save".into();
        }
        _ => {}
    }
}

/// The cloud save and the review events waiting for this player.
fn fetched(w: &mut World, rest: &[&str]) {
    let version: i64 = rest.first().and_then(|v| v.parse().ok()).unwrap_or(0);
    let events = rest.get(1).copied().unwrap_or("");
    let text = rest.get(2).copied().unwrap_or("");
    let take = {
        let c = w.resource::<Cloud>();
        !text.is_empty()
            && (c.take_cloud || if c.synced { version > c.version } else { wall_of(text) > c.local_wall })
    };
    if take {
        adopt(w, text);
    }
    {
        let now = now_ms(w);
        let mut c = w.resource_mut::<Cloud>();
        c.version = version;
        c.step = Step::Ready;
        c.take_cloud = false;
        c.synced = true;
        c.status = "☁️ saved".into();
        // Upload after this tick's save has captured what the events changed.
        c.last_put = now;
    }
    for e in events.split(',').filter(|e| !e.is_empty()) {
        let mut p = e.split(':');
        let (Some(Ok(id)), Some(Ok(n)), Some(Ok(day))) =
            (p.next().map(str::parse::<u64>), p.next().map(str::parse::<u32>), p.next().map(str::parse::<u32>))
        else {
            continue;
        };
        if id <= w.resource::<Town>().study.last_event {
            continue;
        }
        let now = now_ms(w);
        let said = {
            let mut t = w.resource_mut::<Town>();
            t.study.last_event = id;
            t.add_reviews(n, day, now)
        };
        w.resource_mut::<Cloud>().applied.push(id);
        if !said.is_empty() {
            toast(w, format!("📖 {n} Anki reviews · {}", said.join(" · ")));
        }
    }
}

/// The wall time a kept save was made (`…@<wall ms>`), 0 if none.
fn wall_of(text: &str) -> f64 {
    text.rsplit_once('@').and_then(|(_, w)| w.parse().ok()).unwrap_or(0.0)
}

/// Loads a save from the cloud in place of this device's.
fn adopt(w: &mut World, text: &str) {
    let wall = wall_ms(w);
    load(w, &format!("{wall} {text}"));
    w.resource_mut::<Cloud>().dirty = text.to_string();
    w.resource_mut::<Cloud>().sent = text.to_string();
    w.emit(format!("kept {text}"));
}
