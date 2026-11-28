//! Clash of C____, slice 1: the village. Rules live in `economy`; this file
//! draws them, turns taps and HUD buttons into rule calls, and keeps village
//! time (world time plus time spent away).

pub mod battle;
pub mod economy;
pub mod fight;
pub mod cloud;

/// Dev shortcuts (`dev …`, the test `review`) only work after the HUD's hidden
/// 5-tap switch sends `devmode on`. Not saved: off on every fresh start.
#[derive(Default, Resource)]
pub struct DevMode(pub bool);

/// The player's offset from UTC in minutes (the page sends `tz <minutes>`), for the 4 am review day.
#[derive(Default, Resource)]
pub struct Tz(pub i64);

/// Real Unix time in ms, from the host clock the world has seen (0 before it has seen one).
pub fn wall_ms(w: &World) -> u64 {
    let c = w.resource::<Clock>();
    if c.epoch <= 0.0 {
        return 0;
    }
    let ms = c.epoch + (w.tick().saturating_sub(c.epoch_tick)) as f64 * 1000.0 / f64::from(w.hz());
    ms.max(0.0) as u64
}

/// Counts `n` reviews now and tells the player what they earned.
pub fn study(w: &mut World, n: u32) {
    let day = economy::review_day(wall_ms(w), w.resource::<Tz>().0);
    let now = now_ms(w);
    let said = w.resource_mut::<Town>().add_reviews(n, day, now);
    if !said.is_empty() {
        toast(w, said.join(" · "));
    }
}

use economy::{Kind, Pay, Res, Town, Troop, GRID, KINDS, TROOPS};
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
    /// The newest answer from the network (village/data's `supa`), one line.
    #[live]
    pub net: String,
}

#[derive(Default, Resource)]
pub struct SaveContract {
    pub version: u32,
}
pub const SAVE_CONTRACT: u32 = 4;

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
/// Where the camera looks (pan, in tiles from the default spot) and how far
/// back it sits (zoom: 1 is the default, smaller is closer).
#[derive(Default, Resource)]
pub struct CamView {
    pub pan_x: f32,
    pub pan_z: f32,
    pub zoom: f32,
    /// While dragging: the pointer and the ground point it grabbed.
    pub grab: Option<(u64, Vec3)>,
}

impl CamView {
    pub fn home() -> Self {
        CamView { pan_x: 0.0, pan_z: 0.0, zoom: 1.0, grab: None }
    }
}

pub const PAN_LIMIT: f32 = 12.0;
pub const ZOOM_MIN: f32 = 0.45;
pub const ZOOM_MAX: f32 = 1.35;

/// The camera's pose for a view.
pub fn camera_pose(v: &CamView) -> Transform {
    let look = Vec3::new(LOOK.x + v.pan_x, 0.0, LOOK.z + v.pan_z);
    let eye = look + (EYE - LOOK) * v.zoom;
    Transform::at(eye.x, eye.y, eye.z).looking_at(look, Vec3::Y)
}

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
    /// Whether the grid shows its tile lines (only while placing).
    pub grid_lines: bool,
    pub hud: Hud,
    /// The last save sent to the page, and when.
    pub last_save: String,
    pub last_save_ms: u64,
}

/// The save format's version; a save of another version is ignored.
pub const SAVE_PREFIX: &str = "v2";
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
pub struct TrainRow {
    pub key: String,
    pub name: String,
    pub cost: u32,
    pub secs: u32,
    pub housing: u32,
    pub count: u32,
    /// Why it can't be trained now; empty when it can.
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct CampRow {
    pub index: u32,
    pub name: String,
    /// Best stars won here, 0 to 3.
    pub stars: u32,
    pub defenses: u32,
    /// Why it can't be attacked now; empty when it can.
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct DeployRow {
    pub index: u32,
    pub name: String,
    /// Still to drop in this raid.
    pub count: u32,
    pub selected: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Hud {
    /// The network request out now (village/data reads it); "" when none.
    pub ask: String,
    /// Cloud save status for the player.
    pub cloud: String,
    pub gold: u32,
    pub gold_cap: u32,
    pub elixir: u32,
    pub elixir_cap: u32,
    pub gems: u32,
    pub knowledge: u32,
    /// Study today: reviews, the next chest's goal (0 when all opened), the streak, power-hour seconds left.
    pub study_today: u32,
    pub study_next: u32,
    pub streak: u32,
    pub power_secs: u32,
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
    pub has_barracks: bool,
    pub army: Vec<TrainRow>,
    pub housing_used: u32,
    pub housing_cap: u32,
    /// Names of queued troops, the first one training.
    pub queue: Vec<String>,
    pub training_left: u32,
    /// The name of the troop training now; empty when none.
    pub training: String,
    pub campaign: Vec<CampRow>,
    /// In a raid (including its result screen).
    pub raiding: bool,
    pub raid_camp: String,
    pub raid_secs: u32,
    pub raid_percent: u32,
    pub raid_stars: u32,
    pub raid_gold: u32,
    pub raid_elixir: u32,
    pub deploy: Vec<DeployRow>,
    pub has_result: bool,
    pub result_stars: u32,
    pub result_percent: u32,
    pub result_gold: u32,
    pub result_elixir: u32,
    pub result_used: u32,
}

pub const HZ: u32 = 30;
/// Where the camera sits and looks; picking uses the same numbers.
/// A steep corner-on ("diamond") view: 55° down, 45° round, a narrow 35°
/// lens from 60 m away, so the whole 30×30 grid fits a 16:9 view at zoom 1.
pub const EYE: Vec3 = Vec3::new(3.0 + 24.3, 49.1, 3.0 + 24.3);
pub const LOOK: Vec3 = Vec3::new(3.0, 0.0, 3.0);
pub const FOV_Y: f32 = 35.0;

/// The camera's lens; the drawn camera and picking share it.
pub fn lens() -> Camera {
    Camera {
        fov_y_degrees: FOV_Y,
        ..Camera::default()
    }
}

pub struct Village;
impl Game for Village {
    const ID: &'static str = "village";
    const HZ: u32 = HZ;
    const STREAMED: &'static [&'static str] = MODELS;
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
        // Cartoon look: a clear bright sky, fog only far away, saturated grass.
        let sky = Color::hex("#6ec6f5");
        w.insert_resource(Environment {
            background: Some(sky),
            ambient: 0.75,
            fog: Some(Fog {
                color: Some(sky),
                linear: Some([60.0, 110.0]),
                ..Fog::new(0.01, 0.04)
            }),
            ..Environment::default()
        });
        // Meadow, a dirt rim, then the village's buildable grid on top.
        w.spawn((
            Transform::at(0.0, -0.03, 0.0),
            Mesh::plane(120.0, 120.0),
            Material::hex("#4fae2e"),
        ));
        w.spawn((
            Transform::at(0.0, -0.015, 0.0),
            Mesh::plane(GRID as f32 + 1.2, GRID as f32 + 1.2),
            Material::hex("#c9975b"),
        ));
        w.spawn_named(
            "grid",
            (
                Transform::default(),
                Mesh::plane(GRID as f32, GRID as f32),
                grid_look(false),
            ),
        );
        w.spawn_named(
            "camera",
            (
                camera_pose(&CamView::home()),
                lens(),
            ),
        );
        for (name, x, z, k) in SCENERY {
            w.spawn((Transform::at(*x, 0.0, *z).with_scale(*k), Mesh::asset(*name)));
        }
        w.spawn_named(
            "sun",
            (
                Transform::at(12.0, 24.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                DirectionalLight {
                    color: [1.0, 0.94, 0.82],
                    illuminance: 12000.0,
                    shadows: true,
                },
            ),
        );
        w.insert_resource(Town::new());
        w.insert_resource(Clock::default());
        w.insert_resource(Ui::default());
        w.insert_resource(CamView::home());
        w.insert_resource(Shown::default());
        w.insert_resource(fight::Fight::default());
        w.insert_resource(DevMode::default());
        w.insert_resource(Tz::default());
        w.insert_resource(cloud::Cloud::default());
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
        steer(w, input);
        taps(w, input);
        fight::step(w);
        save(w);
        cloud::step(w, &args.net);
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
    let wall = wall_ms(w);
    cloud::kept(w, &format!("{SAVE_PREFIX}|{now}|{town}@{wall}"));
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
    w.resource_mut::<cloud::Cloud>().local_wall = wall_then.parse().unwrap_or(0.0);
    let mut f = body.splitn(3, '|');
    // v1 saves (no army) still load; Town::decode reads both.
    let (Some("v1" | "v2"), Some(then), Some(town)) = (f.next(), f.next(), f.next()) else {
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
    // `cred <email> <password>`: two words, so read before the one-word parse.
    if let Some(rest) = cmd.strip_prefix("cred ") {
        return cloud::set_cred(w, rest);
    }
    let now = now_ms(w);
    let mut words = cmd.split_whitespace();
    let verb = words.next().unwrap_or("");
    let arg = words.next().unwrap_or("");
    match fight::command(w, verb, arg) {
        Err(why) => return toast(w, why),
        Ok(true) => {
            // A raid starts with nothing placed or selected.
            let mut ui = w.resource_mut::<Ui>();
            ui.placing.clear();
            ui.selected = 0;
            return;
        }
        // During a raid, village commands (buy, upgrade, skip…) do nothing;
        // a tap drops the selected troop.
        Ok(false) if fight::active(w) => {
            if verb == "tap" {
                let z = words.next().and_then(|s| s.parse().ok());
                if let (Ok(x), Some(z)) = (arg.parse(), z) {
                    if let Err(why) = fight::deploy_at(w, x, z) {
                        toast(w, why);
                    }
                }
            }
            return;
        }
        Ok(false) => {}
    }
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
                    // Nothing is selected after placing, so a double-click on
                    // Place can't land on "Finish now" and spend gems.
                    let _ = id;
                    ui.selected = 0;
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
        "devmode" => w.resource_mut::<DevMode>().0 = arg == "on",
        "review" | "dev" if !w.resource::<DevMode>().0 => {}
        "review" => study(w, 1),
        "tz" => {
            if let Ok(m) = arg.parse::<i64>() {
                w.resource_mut::<Tz>().0 = m.clamp(-14 * 60, 14 * 60);
            }
        }
        "train" => {
            let Some(t) = Troop::from_key(arg) else { return };
            let done = w.resource_mut::<Town>().train(t, now);
            if let Err(why) = done {
                toast(w, why);
            }
        }
        "untrain" => {
            if let Ok(i) = arg.parse::<usize>() {
                let _ = w.resource_mut::<Town>().cancel_training(i, now);
            }
        }
        // Dev-only shortcuts so checks can reach Town Hall 2-3 content.
        "dev" => match arg {
            "rich" => w.resource_mut::<Town>().dev_rich(),
            "th" => w.resource_mut::<Town>().dev_town_hall(now),
            "army" => w.resource_mut::<Town>().dev_army(now),
            _ => {}
        },
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
                Some((Res::Gold, ..)) => town.gold >= gc && b.waiting_in(now, town.power()) > 0,
                Some((Res::Elixir, ..)) => town.elixir >= ec && b.waiting_in(now, town.power()) > 0,
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

/// Drag pans the camera; the wheel (or a trackpad pinch) zooms. Both stay in bounds.
fn steer(w: &mut World, input: &Input) {
    let view = input.viewport();
    let wheel = input.wheel().y;
    let pose = {
        let mut v = w.resource_mut::<CamView>();
        let before = (v.pan_x, v.pan_z, v.zoom);
        // A drag that ended this tick (even one whole press-move-release inside
        // one tick) finishes with its ground point under where the finger let go.
        let mut ended = Vec::new();
        for e in input.pointer_events() {
            if e.phase != PointerPhase::Up {
                continue;
            }
            ended.push(e.id);
            let origin = input
                .pointers()
                .iter()
                .find(|p| p.id == e.id)
                .map_or(e.position, |p| p.press_origin);
            if origin.distance(e.position) <= 12.0 {
                continue;
            }
            let grabbed = match v.grab {
                Some((id, g)) if id == e.id => Some(g),
                _ => ground_at(view, origin, &v),
            };
            if let (Some(g), Some(now)) = (grabbed, ground_at(view, e.position, &v)) {
                v.pan_x += g.x - now.x;
                v.pan_z += g.z - now.z;
            }
            v.grab = None;
        }
        // The ground point grabbed at the press stays under the finger.
        let drag = input
            .pointers()
            .iter()
            .find(|p| p.down && !ended.contains(&p.id) && p.press_origin.distance(p.position) > 12.0)
            .cloned();
        match drag {
            Some(p) => {
                if v.grab.map(|(id, _)| id) != Some(p.id) {
                    v.grab = ground_at(view, p.press_origin, &v).map(|g| (p.id, g));
                }
                if let (Some((_, g)), Some(now)) = (v.grab, ground_at(view, p.position, &v)) {
                    v.pan_x += g.x - now.x;
                    v.pan_z += g.z - now.z;
                }
            }
            None => v.grab = None,
        }
        if wheel != 0.0 {
            v.zoom = (v.zoom * (1.0 + wheel * 0.0015)).clamp(ZOOM_MIN, ZOOM_MAX);
        }
        v.pan_x = v.pan_x.clamp(-PAN_LIMIT, PAN_LIMIT);
        v.pan_z = v.pan_z.clamp(-PAN_LIMIT, PAN_LIMIT);
        if before == (v.pan_x, v.pan_z, v.zoom) {
            return;
        }
        camera_pose(&v)
    };
    *w.require_mut::<Transform>("camera") = pose;
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
        let hit = tile_at(view, e.position, &w.resource::<CamView>());
        if let Some((x, z)) = hit {
            if fight::active(w) {
                // The second click of a double-click on a camp in the raid list
                // lands on the battlefield: a raid ignores taps for its first
                // third of a second, so it never costs a troop.
                let fresh = w.resource::<fight::Fight>().battle.as_ref().is_some_and(|b| b.tick < 10);
                if fresh {
                    continue;
                }
                if let Err(why) = fight::deploy_at(w, x, z) {
                    toast(w, why);
                }
            } else {
                tap_tile(w, x, z);
            }
        }
    }
}

/// The grid tile under a point on the screen (CSS pixels), if any.
pub fn tile_at(view: Vec2, p: Vec2, v: &CamView) -> Option<(i32, i32)> {
    let hit = ground_at(view, p, v)?;
    let half = GRID as f32 / 2.0;
    let (x, z) = ((hit.x + half).floor() as i32, (hit.z + half).floor() as i32);
    (0..GRID).contains(&x).then_some(())?;
    (0..GRID).contains(&z).then_some(())?;
    Some((x, z))
}

/// The point on the ground (y = 0) under a point on the screen, if any.
pub fn ground_at(view: Vec2, p: Vec2, v: &CamView) -> Option<Vec3> {
    if view.x <= 0.0 || view.y <= 0.0 {
        return None;
    }
    let cam = camera_pose(v);
    let pose = Mat4::from_rotation_translation(cam.rotation, cam.position);
    let inverse = lens().matrix(view).inverse();
    let ndc = Vec3::new(p.x / view.x * 2. - 1., 1. - p.y / view.y * 2., 0.);
    let near = inverse.project_point3(ndc);
    let far = inverse.project_point3(ndc.with_z(1.));
    let origin = pose.transform_point3(Vec3::ZERO);
    let dir = pose.transform_vector3(far - near).normalize();
    if dir.y >= -1e-4 {
        return None;
    }
    let t = -origin.y / dir.y;
    Some(origin + dir * t)
}

/// The world position of a footprint's centre.
fn centre(x: i32, z: i32, size: i32) -> (f32, f32) {
    let half = GRID as f32 / 2.0;
    (
        x as f32 + size as f32 / 2.0 - half,
        z as f32 + size as f32 / 2.0 - half,
    )
}

/// Kenney Castle Kit models (CC0, see art/LICENSE-kenney-castle-kit.txt).
pub const MODELS: &[&str] = &[
    "tower-square.model",
    "tower-hexagon-base.model",
    "tower-hexagon-roof.model",
    "siege-ballista.model",
    "siege-catapult.model",
    "tree-large.model",
    "tree-small.model",
    "rocks-large.model",
    "rocks-small.model",
    "flag.model",
];

/// The model pieces for a kind: (model, height of its base above the stone
/// slab, scale). The models are about one tile wide with their origin at the
/// bottom centre. An empty list means the kind is drawn with plain shapes.
fn pieces(kind: Kind, level: u8) -> Vec<(&'static str, f32, f32)> {
    let grow = 0.85 + 0.15 * f32::from(level.max(1));
    match kind {
        Kind::TownHall => {
            let s = 2.7 * grow;
            vec![
                ("tower-square.model", 0.0, s),
                ("tower-hexagon-roof.model", 1.31 * s, s * 1.05),
                ("flag.model", 1.31 * s + 0.8 * s, s * 0.5),
            ]
        }
        Kind::Barracks => {
            let s = 1.9 * grow;
            vec![("tower-square.model", 0.0, s), ("flag.model", 1.31 * s, s * 0.6)]
        }
        Kind::Cannon => vec![("siege-catapult.model", 0.0, 1.15 * grow)],
        Kind::ArcherTower => {
            let s = 1.6 * grow;
            vec![("tower-hexagon-base.model", 0.0, s), ("siege-ballista.model", 1.31 * s, s * 0.7)]
        }
        Kind::BuilderHut => vec![
            ("tower-hexagon-base.model", 0.0, 1.3),
            ("tower-hexagon-roof.model", 1.31 * 1.3, 1.4),
        ],
        _ => Vec::new(),
    }
}

/// Trees and rocks around the edge of the village (fixed, not saved).
pub const SCENERY: &[(&str, f32, f32, f32)] = &[
    ("tree-large.model", -17.5, -12.0, 2.2),
    ("tree-small.model", -18.5, -6.0, 2.0),
    ("tree-large.model", -17.0, 3.0, 2.4),
    ("rocks-large.model", -18.0, 9.0, 2.0),
    ("tree-small.model", -17.5, 14.0, 2.1),
    ("tree-large.model", 17.5, -14.0, 2.3),
    ("rocks-small.model", 18.0, -7.0, 2.0),
    ("tree-large.model", 17.0, 0.0, 2.2),
    ("tree-small.model", 18.5, 6.0, 2.0),
    ("tree-large.model", 17.5, 12.0, 2.5),
    ("tree-small.model", -12.0, -17.5, 2.0),
    ("tree-large.model", -4.0, -18.0, 2.3),
    ("rocks-large.model", 4.0, -17.5, 1.8),
    ("tree-large.model", 11.0, -18.0, 2.2),
    ("tree-small.model", -9.0, 17.5, 2.0),
    ("rocks-small.model", 0.0, 18.0, 1.8),
    ("tree-large.model", 8.0, 17.5, 2.3),
];

fn colour(kind: Kind) -> &'static str {
    match kind {
        Kind::TownHall => "#b5651d",
        Kind::GoldMine => "#d4a017",
        Kind::ElixirCollector => "#a24bc4",
        Kind::GoldStorage => "#f1c232",
        Kind::ElixirStorage => "#c27ba0",
        Kind::BuilderHut => "#8d6e63",
        Kind::Barracks => "#9e3b2e",
        Kind::ArmyCamp => "#7a6a4f",
        Kind::Cannon => "#555b63",
        Kind::ArcherTower => "#8a7f72",
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
    // Finished buildings with a model draw it; ghosts and first builds keep plain shapes.
    let models = if ghost.is_none() && level > 0 { pieces(kind, level) } else { Vec::new() };
    if !models.is_empty() {
        for (name, y, k) in models {
            parts.push(w.spawn((Transform::at(cx, 0.2 + y, cz).with_scale(k), Mesh::asset(name))));
        }
        let top = 0.2 + 2.0 * s.min(3.0);
        if busy {
            parts.push(w.spawn((
                Transform::at(cx, top, cz),
                Mesh::cuboid(Vec3::new(s - 0.2, 0.15, s - 0.2)),
                Material::hex("#ffd54f").emissive(0.6, 0.45, 0.05),
            )));
        }
        return parts;
    }
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
/// Plain grass, with tile lines only while placing a building.
fn grid_look(lines: bool) -> Material {
    if lines {
        Material::grid(Color::hex("#7fcf45"), 1.0)
    } else {
        Material::hex("#6cc43a")
    }
}

fn draw(w: &mut World) {
    let now = now_ms(w);
    let (selected, placing, gx, gz) = {
        let ui = w.resource::<Ui>();
        (ui.selected, ui.placing.clone(), ui.ghost_x, ui.ghost_z)
    };
    // Material is saved state: write it only when placing starts or stops.
    let lines = !placing.is_empty();
    if w.resource::<Shown>().grid_lines != lines {
        w.resource_mut::<Shown>().grid_lines = lines;
        *w.require_mut::<Material>("grid") = grid_look(lines);
    }
    // Buildings: the village's, or during a raid the camp's (ids from 1000;
    // a destroyed one is rubble, and each sits on its red no-drop border).
    let raid: Option<Vec<(Kind, u8, i32, i32, bool)>> = w
        .resource::<fight::Fight>()
        .battle
        .as_ref()
        .map(|b| b.targets.iter().map(|t| (t.kind, t.level, t.x, t.z, t.alive())).collect());
    let wanted: Vec<(u32, String, Kind, i32, i32, u8, bool, bool)> = match &raid {
        Some(targets) => targets
            .iter()
            .enumerate()
            .map(|(i, &(kind, level, x, z, alive))| {
                (1000 + i as u32, format!("raid{kind:?}{level}{x},{z}{alive}"), kind, x, z, level, false, alive)
            })
            .collect(),
        None => w
            .resource::<Town>()
            .buildings
            .iter()
            .map(|b| {
                let ready = b
                    .kind
                    .production(b.level)
                    .is_some_and(|(_, _, holds)| u64::from(b.waiting_in(now, w.resource::<Town>().power())) * 10 >= holds);
                let sig = format!("{:?}{}{}{}{}", b.kind, b.level, b.busy(), ready, b.id == selected);
                (b.id, sig, b.kind, b.x, b.z, b.level, b.busy(), ready)
            })
            .collect(),
    };
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
        let mut parts = if id >= 1000 {
            // A camp building: `ready` carries whether it still stands.
            let (cx, cz) = centre(x, z, kind.size());
            let s = kind.size() as f32;
            let mut parts = if ready {
                spawn_building(w, kind, x, z, level, false, false, None)
            } else {
                vec![w.spawn((
                    Transform::at(cx, 0.15, cz),
                    Mesh::cuboid(Vec3::new(s - 0.6, 0.3, s - 0.6)),
                    Material::hex("#5d4037"),
                ))]
            };
            parts.push(w.spawn((
                Transform::at(cx, 0.008, cz),
                Mesh::plane(s + 2.0, s + 2.0),
                Material::hex("#e53935").emissive(0.35, 0.05, 0.05),
            )));
            parts
        } else {
            spawn_building(w, kind, x, z, level, busy, ready, None)
        };
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

    fight::draw_units(w);

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
    let today = economy::review_day(wall_ms(w), w.resource::<Tz>().0);
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
                waiting: b.waiting_in(now, town.power()),
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
        ask: w.resource::<cloud::Cloud>().ask.clone(),
        cloud: w.resource::<cloud::Cloud>().status.clone(),
        study_today: if town.study.day == today { town.study.reviews } else { 0 },
        study_next: economy::CHEST_TIERS
            .iter()
            .enumerate()
            .find(|&(i, _)| town.study.day != today || town.study.chests & (1 << i) == 0)
            .map_or(0, |(_, &n)| n),
        streak: town.study.streak_on(today),
        power_secs: (town.study.power_until.saturating_sub(now) / 1000).min(u64::from(u32::MAX)) as u32,
        builders_free: town.builders_free() as u32,
        builders: town.builders() as u32,
        town_hall: u32::from(th),
        placing: Kind::from_key(&ui.placing).map(|k| k.name().to_string()).unwrap_or_default(),
        place_reason,
        shop,
        has_selected: sel.is_some(),
        selected,
        toast: if now < ui.toast_until { ui.toast.clone() } else { String::new() },
        has_barracks: town.barracks_level() > 0,
        army: TROOPS
            .iter()
            .map(|&t| TrainRow {
                key: t.key().into(),
                name: t.name().into(),
                cost: t.cost(),
                secs: t.train_secs() as u32,
                housing: t.housing(),
                count: town.trained(t),
                reason: town.can_train(t).err().unwrap_or_default(),
            })
            .collect(),
        housing_used: town.housing_used(),
        housing_cap: town.housing_cap(),
        queue: town.queue.iter().map(|t| t.name().to_string()).collect(),
        training_left: town.training_left(now) as u32,
        training: town.queue.first().map(|t| t.name().to_string()).unwrap_or_default(),
        campaign: battle::CAMPS
            .iter()
            .enumerate()
            .map(|(i, (name, plan))| CampRow {
                index: i as u32,
                name: (*name).into(),
                stars: u32::from(town.stars.get(i).copied().unwrap_or(0)),
                defenses: plan.iter().filter(|p| p.kind.is_defense()).count() as u32,
                reason: fight::can_attack(w, i).err().unwrap_or_default(),
            })
            .collect(),
        ..raid_hud(w)
    }
}

/// The raid part of the HUD (all zero outside a raid).
fn raid_hud(w: &World) -> Hud {
    let f = w.resource::<fight::Fight>();
    let Some(b) = &f.battle else { return Hud::default() };
    let r = f.result.clone().unwrap_or_default();
    Hud {
        raiding: true,
        raid_camp: battle::CAMPS[(b.camp as usize).min(battle::CAMPS.len() - 1)].0.into(),
        raid_secs: b.ticks_left().div_ceil(battle::TPS),
        raid_percent: b.percent(),
        raid_stars: u32::from(b.stars()),
        raid_gold: b.gold,
        raid_elixir: b.elixir,
        deploy: TROOPS
            .iter()
            .map(|&t| DeployRow {
                index: t.index() as u32,
                name: t.name().into(),
                count: b.reserve.get(t.index()).copied().unwrap_or(0),
                selected: f.troop as usize == t.index(),
            })
            .collect(),
        has_result: f.result.is_some(),
        result_stars: u32::from(r.stars),
        result_percent: r.percent,
        result_gold: r.gold,
        result_elixir: r.elixir,
        result_used: r.used,
        ..Hud::default()
    }
}
