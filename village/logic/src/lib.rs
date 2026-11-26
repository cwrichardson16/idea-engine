//! Clash of C____, slice 1: the village. Rules live in `economy`; this file
//! draws them, turns taps and HUD buttons into rule calls, and keeps village
//! time (world time plus time spent away).

pub mod economy;

use economy::{Kind, Pay, Res, Town, GRID, KINDS};
use exact_game::*;

#[derive(Default, Args)]
pub struct Options {
    pub seed: u64,
    #[live]
    pub paused: bool,
    #[restart]
    pub restart: bool,
    /// The host's Unix time (ms) at clock zero; a jump means time passed away.
    #[live]
    pub epoch: f64,
}

#[derive(Default, Resource)]
pub struct SaveContract {
    pub version: u32,
}
pub const SAVE_CONTRACT: u32 = 1;

/// Village time: world time plus every span spent away.
#[derive(Default, Resource)]
pub struct Clock {
    pub offline_ms: u64,
    pub epoch: f64,
    pub epoch_tick: u64,
}

/// What the player is doing: placing a new building (and where its ghost
/// sits), or looking at a selected one.
#[derive(Default, Resource)]
pub struct Ui {
    /// The kind being placed, by key; empty when not placing.
    pub placing: String,
    pub ghost_x: i32,
    pub ghost_z: i32,
    /// The selected building's id; 0 for none.
    pub selected: u32,
    pub toast: String,
    pub toast_until: u64,
    /// True once the page has sent what it kept (or that it kept nothing);
    /// saving waits for this so a fresh world never overwrites the real village.
    pub loaded: bool,
}

/// What is drawn for each building, so it is redrawn only when it changes.
#[derive(Clone, Default, PartialEq, Data)]
pub struct Drawn {
    pub id: u32,
    pub sig: String,
    pub parts: Vec<Entity>,
}
#[derive(Default, Resource)]
pub struct Shown {
    pub buildings: Vec<Drawn>,
    pub ghost: Vec<Entity>,
    pub ghost_sig: String,
    pub hud: Hud,
    /// The last save sent to the page, and when.
    pub last_save: String,
    pub last_save_ms: u64,
}

/// The save format's version; a save of another version is ignored.
pub const SAVE_PREFIX: &str = "v1";
/// Even with no changes, re-save this often so time away is measured well.
pub const SAVE_EVERY_MS: u64 = 10_000;

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct ShopRow {
    pub key: String,
    pub name: String,
    pub cost: u32,
    /// "gold" or "elixir".
    pub res: String,
    pub secs: u32,
    pub owned: u32,
    pub max: u32,
    /// Why it can't be bought now; empty when it can.
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Selected {
    pub id: u32,
    pub name: String,
    pub level: u32,
    pub max_level: u32,
    /// Seconds left on its builder's timer; 0 when idle.
    pub left: u32,
    /// What is waiting to be collected, and its resource.
    pub waiting: u32,
    pub res: String,
    pub upgrade_cost: u32,
    pub upgrade_res: String,
    pub upgrade_secs: u32,
    pub upgrade_reason: String,
    pub skip_gems: u32,
    pub skip_knowledge: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Hud {
    pub gold: u32,
    pub gold_cap: u32,
    pub elixir: u32,
    pub elixir_cap: u32,
    pub gems: u32,
    pub knowledge: u32,
    pub builders_free: u32,
    pub builders: u32,
    pub town_hall: u32,
    /// The name of what is being placed; empty when not placing.
    pub placing: String,
    /// Why the ghost can't go where it is; empty when it can.
    pub place_reason: String,
    pub shop: Vec<ShopRow>,
    pub has_selected: bool,
    pub selected: Selected,
    pub toast: String,
}

pub const HZ: u32 = 30;
/// Where the camera sits and looks; picking uses the same numbers.
pub const EYE: Vec3 = Vec3::new(0.0, 24.0, 22.0);
pub const LOOK: Vec3 = Vec3::new(0.0, 0.0, 1.5);

pub struct Village;
impl Game for Village {
    const ID: &'static str = "village";
    const HZ: u32 = HZ;
    type Args = Options;

    fn restorable(saved: &World) -> Result<(), String> {
        match saved.try_resource::<SaveContract>().map(|c| c.version) {
            Some(SAVE_CONTRACT) => Ok(()),
            found => Err(format!(
                "its save contract is {found:?}; this build reads version {SAVE_CONTRACT}"
            )),
        }
    }

    fn setup(w: &mut World, args: &Options) {
        w.reseed(args.seed);
        w.insert_resource(SaveContract {
            version: SAVE_CONTRACT,
        });
        let sky = Color::hex("#a8d8ea");
        w.insert_resource(Environment {
            background: Some(sky),
            fog: Some(Fog {
                color: Some(sky),
                ..Fog::new(0.01, 0.04)
            }),
            ..Environment::default()
        });
        // Meadow, then the village's buildable grid on top.
        w.spawn((
            Transform::at(0.0, -0.02, 0.0),
            Mesh::plane(120.0, 120.0),
            Material::hex("#6aa84f"),
        ));
        w.spawn((
            Transform::default(),
            Mesh::plane(GRID as f32, GRID as f32),
            Material::grid(Color::hex("#93c47d"), 1.0),
        ));
        w.spawn_named(
            "camera",
            (
                Transform::at(EYE.x, EYE.y, EYE.z).looking_at(LOOK, Vec3::Y),
                Camera::default(),
            ),
        );
        w.spawn_named(
            "sun",
            (
                Transform::at(12.0, 24.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                DirectionalLight::default(),
            ),
        );
        w.insert_resource(Town::new());
        w.insert_resource(Clock::default());
        w.insert_resource(Ui::default());
        w.insert_resource(Shown::default());
        draw(w);
    }

    fn paused(args: &Options) -> bool {
        args.paused
    }

    fn tick(w: &mut World, input: &Input, args: &Options) {
        observe_epoch(w, args.epoch);
        let now = now_ms(w);
        w.resource_mut::<Town>().settle(now);
        let commands: Vec<String> = input.messages().iter().map(|m| m.to_string()).collect();
        for cmd in commands {
            if let Some(rest) = cmd.strip_prefix("load") {
                load(w, rest.trim());
            } else {
                command(w, &cmd);
            }
        }
        taps(w, input);
        save(w);
        if w.resource::<Town>().builders_busy() > 0 {
            w.busy("builders at work");
        }
        draw(w);
    }
}

/// Village milliseconds: world time plus time away.
pub fn now_ms(w: &World) -> u64 {
    w.tick() * 1000 / u64::from(w.hz()) + w.resource::<Clock>().offline_ms
}

/// Lets `ms` pass at once, as if the player had been away.
pub fn away(w: &mut World, ms: u64) {
    w.resource_mut::<Clock>().offline_ms += ms;
    let now = now_ms(w);
    w.resource_mut::<Town>().settle(now);
}

/// Sends the village to the page to keep, when it changed or every few
/// seconds. Format: `save v1|<village ms>|<town>`; the page appends
/// `@<real wall-clock ms>` when it keeps it.
fn save(w: &mut World) {
    if !w.resource::<Ui>().loaded {
        return;
    }
    let now = now_ms(w);
    let town = w.resource::<Town>().encode();
    let (changed, due) = {
        let s = w.resource::<Shown>();
        let old_town = s.last_save.rsplit('|').next().unwrap_or("");
        (old_town != town, now >= s.last_save_ms + SAVE_EVERY_MS)
    };
    if !changed && !due {
        return;
    }
    let text = format!("save {SAVE_PREFIX}|{now}|{town}");
    w.emit(text.clone());
    let mut s = w.resource_mut::<Shown>();
    s.last_save = text;
    s.last_save_ms = now;
}

/// Restores what the page kept. The page sends `<real wall ms now>` and then
/// what it kept, `v1|<village ms>|<town>@<real wall ms then>` (or nothing).
/// The real time between the two is time away. Anything unreadable starts fresh.
pub fn load(w: &mut World, text: &str) {
    w.resource_mut::<Ui>().loaded = true;
    let mut parts = text.splitn(2, ' ');
    let wall_now: f64 = parts.next().and_then(|n| n.parse().ok()).unwrap_or(0.0);
    let kept = parts.next().unwrap_or("").trim();
    let Some((body, wall_then)) = kept.rsplit_once('@') else { return };
    let mut f = body.splitn(3, '|');
    let (Some(SAVE_PREFIX), Some(then), Some(town)) = (f.next(), f.next(), f.next()) else {
        return;
    };
    let (Ok(then), Ok(wall_then), Some(town)) =
        (then.parse::<u64>(), wall_then.parse::<f64>(), Town::decode(town))
    else {
        return;
    };
    w.insert_resource(town);
    // Continue village time from where it stopped, then add the real time away.
    let tick_ms = w.tick() * 1000 / u64::from(w.hz());
    w.resource_mut::<Clock>().offline_ms = then.saturating_sub(tick_ms);
    let gap = wall_now - wall_then;
    if gap >= 1000.0 && gap.is_finite() {
        away(w, gap as u64);
    }
    let mut ui = w.resource_mut::<Ui>();
    ui.selected = 0;
    ui.placing.clear();
}

/// A later session's wall clock, ahead of what this world believes, is time away.
fn observe_epoch(w: &mut World, epoch: f64) {
    if epoch <= 0.0 {
        return;
    }
    let (seen, seen_tick) = {
        let c = w.resource::<Clock>();
        (c.epoch, c.epoch_tick)
    };
    if seen == epoch {
        return;
    }
    if seen > 0.0 {
        let believed =
            seen + (w.tick() - seen_tick.min(w.tick())) as f64 * 1000.0 / f64::from(w.hz());
        let gap = epoch - believed;
        if gap >= 1000.0 {
            away(w, gap as u64);
        }
    }
    let tick = w.tick();
    let mut c = w.resource_mut::<Clock>();
    c.epoch = epoch;
    c.epoch_tick = tick;
}

fn toast(w: &mut World, msg: impl Into<String>) {
    let until = now_ms(w) + 2500;
    let mut ui = w.resource_mut::<Ui>();
    ui.toast = msg.into();
    ui.toast_until = until;
}

/// One HUD command: `buy goldmine`, `place`, `cancel`, `upgrade`, `collect`,
/// `collectall`, `skip gems`, `skip knowledge`, `review`, `away 60`, `tap 12 7`.
pub fn command(w: &mut World, cmd: &str) {
    let now = now_ms(w);
    let mut words = cmd.split_whitespace();
    let verb = words.next().unwrap_or("");
    let arg = words.next().unwrap_or("");
    let selected = w.resource::<Ui>().selected;
    match verb {
        "buy" => {
            let Some(kind) = Kind::from_key(arg) else { return };
            let can = w.resource::<Town>().can_buy(kind);
            if let Err(why) = can {
                return toast(w, why);
            }
            let (x, z) = w
                .resource::<Town>()
                .free_spot(kind)
                .unwrap_or(((GRID - kind.size()) / 2, 0));
            let mut ui = w.resource_mut::<Ui>();
            ui.placing = kind.key().into();
            ui.ghost_x = x;
            ui.ghost_z = z;
            ui.selected = 0;
        }
        "place" => {
            let (key, x, z) = {
                let ui = w.resource::<Ui>();
                (ui.placing.clone(), ui.ghost_x, ui.ghost_z)
            };
            let Some(kind) = Kind::from_key(&key) else { return };
            let placed = w.resource_mut::<Town>().place(kind, x, z, now);
            match placed {
                Ok(id) => {
                    let mut ui = w.resource_mut::<Ui>();
                    ui.placing.clear();
                    ui.selected = id;
                }
                Err(why) => toast(w, why),
            }
        }
        "cancel" => {
            let mut ui = w.resource_mut::<Ui>();
            ui.placing.clear();
            ui.selected = 0;
        }
        "upgrade" => {
            let done = w.resource_mut::<Town>().upgrade(selected, now);
            if let Err(why) = done {
                toast(w, why);
            }
        }
        "collect" => {
            w.resource_mut::<Town>().collect(selected, now);
        }
        "collectall" => {
            w.resource_mut::<Town>().collect_all(now);
        }
        "skip" => {
            let pay = if arg == "gems" { Pay::Gems } else { Pay::Knowledge };
            let done = w.resource_mut::<Town>().skip(selected, now, pay);
            if let Err(why) = done {
                toast(w, why);
            }
        }
        "review" => w.resource_mut::<Town>().review(),
        "away" => {
            if let Ok(s) = arg.parse::<u64>() {
                away(w, s * 1000);
            }
        }
        "tap" => {
            let z = words.next().and_then(|s| s.parse().ok());
            if let (Ok(x), Some(z)) = (arg.parse(), z) {
                tap_tile(w, x, z);
            }
        }
        _ => {}
    }
}

/// A tap on tile (x, z): move the ghost there while placing; otherwise
/// select what's there and collect from it.
pub fn tap_tile(w: &mut World, x: i32, z: i32) {
    let now = now_ms(w);
    let placing = Kind::from_key(&w.resource::<Ui>().placing);
    if let Some(kind) = placing {
        let s = kind.size();
        let mut ui = w.resource_mut::<Ui>();
        ui.ghost_x = (x - s / 2).clamp(0, GRID - s);
        ui.ghost_z = (z - s / 2).clamp(0, GRID - s);
        return;
    }
    let hit = w.resource::<Town>().at(x, z).map(|b| b.id).unwrap_or(0);
    if hit != 0 {
        let full = {
            let town = w.resource::<Town>();
            let (gc, ec) = town.caps();
            town.get(hit).is_some_and(|b| match b.kind.production(b.level) {
                Some((Res::Gold, ..)) => town.gold >= gc && b.waiting(now) > 0,
                Some((Res::Elixir, ..)) => town.elixir >= ec && b.waiting(now) > 0,
                None => false,
            })
        };
        let got = w.resource_mut::<Town>().collect(hit, now);
        if full {
            toast(w, "Storage full: build or upgrade storage");
        } else if got > 0 {
            toast(w, format!("+{got}"));
        }
    }
    w.resource_mut::<Ui>().selected = hit;
}

/// Turns pointer taps on the world into tile taps. A press that moved more
/// than a few pixels is a drag, not a tap.
fn taps(w: &mut World, input: &Input) {
    let view = input.viewport();
    for e in input.pointer_events() {
        if e.phase != PointerPhase::Up {
            continue;
        }
        let origin = input
            .pointers()
            .iter()
            .find(|p| p.id == e.id)
            .map_or(e.position, |p| p.press_origin);
        if origin.distance(e.position) > 12.0 {
            continue;
        }
        if let Some((x, z)) = tile_at(view, e.position) {
            tap_tile(w, x, z);
        }
    }
}

/// The grid tile under a point on the screen (CSS pixels), if any.
pub fn tile_at(view: Vec2, p: Vec2) -> Option<(i32, i32)> {
    if view.x <= 0.0 || view.y <= 0.0 {
        return None;
    }
    let cam = Transform::at(EYE.x, EYE.y, EYE.z).looking_at(LOOK, Vec3::Y);
    let pose = Mat4::from_rotation_translation(cam.rotation, cam.position);
    let inverse = Camera::default().matrix(view).inverse();
    let ndc = Vec3::new(p.x / view.x * 2. - 1., 1. - p.y / view.y * 2., 0.);
    let near = inverse.project_point3(ndc);
    let far = inverse.project_point3(ndc.with_z(1.));
    let origin = pose.transform_point3(Vec3::ZERO);
    let dir = pose.transform_vector3(far - near).normalize();
    if dir.y >= -1e-4 {
        return None;
    }
    let t = -origin.y / dir.y;
    let hit = origin + dir * t;
    let half = GRID as f32 / 2.0;
    let (x, z) = ((hit.x + half).floor() as i32, (hit.z + half).floor() as i32);
    (0..GRID).contains(&x).then_some(())?;
    (0..GRID).contains(&z).then_some(())?;
    Some((x, z))
}

/// The world position of a footprint's centre.
fn centre(x: i32, z: i32, size: i32) -> (f32, f32) {
    let half = GRID as f32 / 2.0;
    (
        x as f32 + size as f32 / 2.0 - half,
        z as f32 + size as f32 / 2.0 - half,
    )
}

fn colour(kind: Kind) -> &'static str {
    match kind {
        Kind::TownHall => "#b5651d",
        Kind::GoldMine => "#d4a017",
        Kind::ElixirCollector => "#a24bc4",
        Kind::GoldStorage => "#f1c232",
        Kind::ElixirStorage => "#c27ba0",
        Kind::BuilderHut => "#8d6e63",
    }
}

/// Spawns one building's parts: a stone base, a body that grows with level,
/// a roof, a yellow scaffold cap while builders work, and a glowing bubble
/// when there's plenty to collect.
fn spawn_building(w: &mut World, kind: Kind, x: i32, z: i32, level: u8, busy: bool, ready: bool, ghost: Option<bool>) -> Vec<Entity> {
    let s = kind.size() as f32;
    let (cx, cz) = centre(x, z, kind.size());
    let mut parts = Vec::new();
    let tint = |hex: &str| match ghost {
        Some(true) => "#7ee787".to_string(),
        Some(false) => "#ff6b6b".to_string(),
        None => hex.to_string(),
    };
    // A blocked ghost floats above whatever is in the way, so it can't hide inside it.
    let lift = if ghost == Some(false) { 2.8 } else { 0.0 };
    parts.push(w.spawn((
        Transform::at(cx, lift + 0.1, cz),
        Mesh::cuboid(Vec3::new(s - 0.15, 0.2, s - 0.15)),
        Material::hex(&tint("#7f7f7f")),
    )));
    let h = if level == 0 { 0.35 } else { 0.5 + 0.45 * f32::from(level) };
    let body = if level == 0 { "#bdb5a6" } else { colour(kind) };
    parts.push(w.spawn((
        Transform::at(cx, lift + 0.2 + h / 2.0, cz),
        Mesh::cuboid(Vec3::new(s - 0.7, h, s - 0.7)),
        Material::hex(&tint(body)),
    )));
    if level > 0 {
        parts.push(w.spawn((
            Transform::at(cx, lift + 0.2 + h + 0.2, cz),
            Mesh::cuboid(Vec3::new(s - 0.4, 0.4, s - 0.4)),
            Material::hex(&tint("#6d4c41")),
        )));
    }
    if busy {
        parts.push(w.spawn((
            Transform::at(cx, 0.2 + h + 0.55, cz),
            Mesh::cuboid(Vec3::new(s - 0.2, 0.15, s - 0.2)),
            Material::hex("#ffd54f").emissive(0.6, 0.45, 0.05),
        )));
    }
    if ready {
        parts.push(w.spawn((
            Transform::at(cx, 0.2 + h + 1.2, cz),
            Mesh::sphere(0.35),
            Material::hex(colour(kind)).emissive(1.5, 1.2, 0.3),
        )));
    }
    parts
}

/// Redraws what changed and publishes the HUD when it changed.
fn draw(w: &mut World) {
    let now = now_ms(w);
    let (selected, placing, gx, gz) = {
        let ui = w.resource::<Ui>();
        (ui.selected, ui.placing.clone(), ui.ghost_x, ui.ghost_z)
    };
    // Buildings.
    let wanted: Vec<(u32, String, Kind, i32, i32, u8, bool, bool)> = w
        .resource::<Town>()
        .buildings
        .iter()
        .map(|b| {
            let ready = b
                .kind
                .production(b.level)
                .is_some_and(|(_, _, holds)| u64::from(b.waiting(now)) * 10 >= holds);
            let sig = format!("{:?}{}{}{}{}", b.kind, b.level, b.busy(), ready, b.id == selected);
            (b.id, sig, b.kind, b.x, b.z, b.level, b.busy(), ready)
        })
        .collect();
    let mut shown = std::mem::take(&mut w.resource_mut::<Shown>().buildings);
    shown.retain(|d| {
        let keep = wanted.iter().any(|(id, sig, ..)| *id == d.id && *sig == d.sig);
        if !keep {
            for &e in &d.parts {
                w.despawn(e);
            }
        }
        keep
    });
    for (id, sig, kind, x, z, level, busy, ready) in wanted {
        if shown.iter().any(|d| d.id == id) {
            continue;
        }
        let mut parts = spawn_building(w, kind, x, z, level, busy, ready, None);
        if id == selected {
            let (cx, cz) = centre(x, z, kind.size());
            let s = kind.size() as f32;
            parts.push(w.spawn((
                Transform::at(cx, 0.02, cz),
                Mesh::plane(s + 0.3, s + 0.3),
                Material::hex("#fff59d").emissive(0.8, 0.8, 0.3),
            )));
        }
        shown.push(Drawn { id, sig, parts });
    }
    w.resource_mut::<Shown>().buildings = shown;

    // The ghost of what's being placed: green where it fits, red where it doesn't.
    let place_reason = match Kind::from_key(&placing) {
        Some(kind) => w.resource::<Town>().can_place(kind, gx, gz).err().unwrap_or_default(),
        None => String::new(),
    };
    let ghost_sig = format!("{placing}{gx},{gz}{}", place_reason.is_empty());
    if w.resource::<Shown>().ghost_sig != ghost_sig {
        let old = std::mem::take(&mut w.resource_mut::<Shown>().ghost);
        for e in old {
            w.despawn(e);
        }
        let ghost = match Kind::from_key(&placing) {
            Some(kind) => {
                let fits = place_reason.is_empty();
                let mut parts = spawn_building(w, kind, gx, gz, 1, false, false, Some(fits));
                // Its footprint on the ground, always visible.
                let (cx, cz) = centre(gx, gz, kind.size());
                let s = kind.size() as f32;
                parts.push(w.spawn((
                    Transform::at(cx, 0.04, cz),
                    Mesh::plane(s, s),
                    if fits {
                        Material::hex("#7ee787").emissive(0.2, 0.6, 0.2)
                    } else {
                        Material::hex("#ff6b6b").emissive(0.8, 0.15, 0.15)
                    },
                )));
                parts
            }
            None => Vec::new(),
        };
        let mut s = w.resource_mut::<Shown>();
        s.ghost = ghost;
        s.ghost_sig = ghost_sig;
    }

    // The HUD.
    let hud = hud(w, now, place_reason);
    if w.resource::<Shown>().hud != hud {
        w.publish_record(&hud);
        w.resource_mut::<Shown>().hud = hud;
    }
}

fn res_name(r: Res) -> String {
    match r {
        Res::Gold => "gold".into(),
        Res::Elixir => "elixir".into(),
    }
}

/// The HUD as the player sees it now.
pub fn hud(w: &World, now: u64, place_reason: String) -> Hud {
    let town = w.resource::<Town>();
    let ui = w.resource::<Ui>();
    let (gold_cap, elixir_cap) = town.caps();
    let th = town.town_hall_level();
    let shop = KINDS
        .iter()
        .filter(|k| **k != Kind::TownHall)
        .map(|&k| {
            let (r, cost) = k.cost(1);
            ShopRow {
                key: k.key().into(),
                name: k.name().into(),
                cost,
                res: res_name(r),
                secs: k.build_secs(1) as u32,
                owned: town.count(k) as u32,
                max: k.max_count(th) as u32,
                reason: town.can_buy(k).err().unwrap_or_default(),
            }
        })
        .collect();
    let sel = town.get(ui.selected);
    let selected = sel
        .map(|b| {
            let next = b.level + 1;
            let (ur, uc) = b.kind.cost(next);
            let maxed = b.level >= b.kind.max_level();
            Selected {
                id: b.id,
                name: b.kind.name().into(),
                level: u32::from(b.level),
                max_level: u32::from(b.kind.max_level()),
                left: if b.busy() {
                    b.done_at.saturating_sub(now).div_ceil(1000) as u32
                } else {
                    0
                },
                waiting: b.waiting(now),
                res: b
                    .kind
                    .production(b.level.max(1))
                    .map(|(r, ..)| res_name(r))
                    .unwrap_or_default(),
                upgrade_cost: if maxed { 0 } else { uc },
                upgrade_res: res_name(ur),
                upgrade_secs: if maxed { 0 } else { b.kind.build_secs(next) as u32 },
                upgrade_reason: town.can_upgrade(b.id).err().unwrap_or_default(),
                skip_gems: town.skip_cost(b.id, now, Pay::Gems).unwrap_or(0),
                skip_knowledge: town.skip_cost(b.id, now, Pay::Knowledge).unwrap_or(0),
            }
        })
        .unwrap_or_default();
    Hud {
        gold: town.gold,
        gold_cap,
        elixir: town.elixir,
        elixir_cap,
        gems: town.gems,
        knowledge: town.knowledge,
        builders_free: town.builders_free() as u32,
        builders: town.builders() as u32,
        town_hall: u32::from(th),
        placing: Kind::from_key(&ui.placing).map(|k| k.name().to_string()).unwrap_or_default(),
        place_reason,
        shop,
        has_selected: sel.is_some(),
        selected,
        toast: if now < ui.toast_until { ui.toast.clone() } else { String::new() },
    }
}
