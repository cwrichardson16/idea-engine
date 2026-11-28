//! A raid in the world: start it, drop troops where the player taps, step it
//! on the village's 30 Hz tick, and settle the result into the village.
//! The battle itself is never saved: troops leave the army as they are
//! dropped, and loot only arrives when the battle ends.

use crate::battle::*;
use crate::economy::*;
use exact_game::*;

#[derive(Default, Resource)]
pub struct Fight {
    /// The raid in progress, kept while its result shows.
    pub battle: Option<Battle>,
    /// The troop kind the next tap drops, by `TROOPS` index.
    pub troop: u32,
    pub result: Option<RaidResult>,
    /// Drawn troops, one per battle unit.
    pub units: Vec<Entity>,
    /// What each drawn Spearman is playing (`Act`), and where it stood last draw.
    pub acts: Vec<(Act, i64, i64)>,
    /// HP bars over damaged troops and buildings (drawing only).
    pub bars: Vec<Bar>,
    /// Arrows and cannonballs in flight (drawing only).
    pub flying: Vec<Flying>,
    /// Shots drawn this raid, and the battle tick whose shots were last drawn.
    pub shots_drawn: u32,
    pub shots_tick: u32,
}

/// One HP bar: a dark back and a coloured fill, over troop `index` (`unit`)
/// or building `index`.
#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Bar {
    pub unit: bool,
    pub index: u32,
    pub back: Entity,
    pub fill: Entity,
}

/// One shot in the air: it leaves `from` at battle tick `born` and lands on
/// `to` `ticks` later, on a low arc.
#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Flying {
    pub entity: Entity,
    pub from: Vec3,
    pub to: Vec3,
    pub born: u32,
    pub ticks: u32,
    pub ball: bool,
}

/// A Spearman's clip: walking to its target, swinging at it, or fallen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Data)]
pub enum Act {
    #[default]
    Idle,
    Walk,
    Attack,
    Dead,
}

/// The Spearman's model and its scale (the KayKit Knight is about 1 m tall at
/// scale 1, chunky and big-headed; 0.9 reads at the steep village camera).
pub const SPEARMAN_MODEL: &str = "knight.model";
const SPEARMAN_SCALE: f32 = 0.9;

fn clip(act: Act) -> Animation {
    match act {
        Act::Idle => Animation::play("Idle"),
        Act::Walk => Animation::play("Walking_A").speed(1.2),
        Act::Attack => Animation::play("1H_Melee_Attack_Stab"),
        Act::Dead => Animation::play("Death_A").once(),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct RaidResult {
    pub camp: u32,
    pub stars: u8,
    pub percent: u32,
    /// Loot actually kept (after storage caps).
    pub gold: u32,
    pub elixir: u32,
    pub used: u32,
}

/// In a raid or looking at its result.
pub fn active(w: &World) -> bool {
    w.resource::<Fight>().battle.is_some()
}

/// Why a raid on `camp` can't start now; Ok when it can.
pub fn can_attack(w: &World, camp: usize) -> Result<(), String> {
    if camp >= CAMPS.len() {
        return Err("No such camp".into());
    }
    if active(w) {
        return Err("Already raiding".into());
    }
    if w.resource::<Town>().army.iter().all(|&n| n == 0) {
        return Err("Train some troops first".into());
    }
    Ok(())
}

/// Battle commands: `attack <camp>`, `troop <index>`, `surrender`, `home`.
/// Returns whether the command was one.
pub fn command(w: &mut World, verb: &str, arg: &str) -> Result<bool, String> {
    match verb {
        "attack" => {
            let camp: usize = arg.parse().map_err(|_| "No such camp".to_string())?;
            can_attack(w, camp)?;
            let town = w.resource::<Town>();
            let reserve = town.army.clone();
            let looted = town.looted.get(camp).copied().unwrap_or(0);
            let (keep, troop_level) = (town.town_hall_level(), town.barracks_level());
            let first = reserve.iter().position(|&n| n > 0).unwrap_or(0) as u32;
            drop(town);
            let mut f = w.resource_mut::<Fight>();
            f.battle = Some(Battle::camp(camp, keep, troop_level, reserve, looted));
            f.troop = first;
            f.result = None;
            f.shots_drawn = 0;
            f.shots_tick = 0;
            Ok(true)
        }
        "troop" => {
            let i: usize = arg.parse().map_err(|_| "No such troop".to_string())?;
            if i < TROOPS.len() {
                w.resource_mut::<Fight>().troop = i as u32;
            }
            Ok(true)
        }
        "surrender" => {
            if let Some(b) = w.resource_mut::<Fight>().battle.as_mut() {
                b.surrender();
            }
            Ok(true)
        }
        "home" => {
            let mut f = w.resource_mut::<Fight>();
            if f.result.is_some() {
                f.battle = None;
                f.result = None;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// A tap on tile (x, z) during a raid drops the selected troop there.
pub fn deploy_at(w: &mut World, x: i32, z: i32) -> Result<(), String> {
    let mut f = w.resource_mut::<Fight>();
    let t = TROOPS[(f.troop as usize).min(TROOPS.len() - 1)];
    let Some(b) = f.battle.as_mut() else { return Ok(()) };
    b.deploy(t, x, z)?;
    // Out of this kind: pick the next kind still in reserve.
    if b.reserve[t.index()] == 0 {
        if let Some(next) = b.reserve.iter().position(|&n| n > 0) {
            f.troop = next as u32;
        }
    }
    drop(f);
    w.resource_mut::<Town>().take_troops(t, 1);
    Ok(())
}

/// One tick of the raid; settles the village once when it ends.
pub fn step(w: &mut World) {
    let mut f = w.resource_mut::<Fight>();
    if f.result.is_some() {
        return;
    }
    let Some(b) = f.battle.as_mut() else { return };
    if !b.over {
        b.step();
    }
    if !b.over {
        drop(f);
        w.busy("battle");
        return;
    }
    let (camp, stars, percent, looted, gold, elixir, used) = (
        b.camp,
        b.stars(),
        b.percent(),
        b.looted_now(),
        b.gold,
        b.elixir,
        b.deployed,
    );
    drop(f);
    let (gold, elixir) = w.resource_mut::<Town>().record_raid(camp as usize, stars, looted, gold, elixir);
    w.resource_mut::<Fight>().result = Some(RaidResult { camp, stars, percent, gold, elixir, used });
}

fn troop_colour(t: Troop) -> &'static str {
    match t {
        Troop::Fighter => "#ffb74d",
        Troop::Archer => "#ba68c8",
        Troop::Brute => "#8d6e63",
    }
}

/// Draws the raid's troops. The Spearman is the animated KayKit Knight
/// (walk, stab, die); the others are still a capsule each, nudged apart by
/// index so a crowd hitting one building doesn't look like one blob, and
/// sink when they fall. Call `animation::step` after this.
pub fn draw_units(w: &mut World) {
    let half = GRID as f32 / 2.0;
    let units: Vec<(Troop, i64, i64, bool, Option<(f32, f32)>)> = match &w.resource::<Fight>().battle {
        Some(b) => b
            .units
            .iter()
            .map(|u| {
                let aim = u.target.map(|t| {
                    let (cx, cz) = b.targets[t as usize].centre();
                    (cx as f32 / MILLI as f32 - half, cz as f32 / MILLI as f32 - half)
                });
                (u.troop, u.x, u.z, u.alive(), aim)
            })
            .collect(),
        None => Vec::new(),
    };
    let mut drawn = std::mem::take(&mut w.resource_mut::<Fight>().units);
    let mut acts = std::mem::take(&mut w.resource_mut::<Fight>().acts);
    while drawn.len() > units.len() {
        let e = drawn.pop().unwrap();
        w.despawn(e);
    }
    acts.truncate(units.len());
    for (i, (troop, x, z, alive, aim)) in units.into_iter().enumerate() {
        let (ox, oz) = ((i * 7 % 5) as f32 * 0.12 - 0.24, (i * 3 % 7) as f32 * 0.08 - 0.24);
        let (px, pz) = (x as f32 / MILLI as f32 - half + ox, z as f32 / MILLI as f32 - half + oz);
        if troop == Troop::Fighter {
            let (act, was_x, was_z) = acts.get(i).copied().unwrap_or((Act::Idle, x, z));
            let now = if !alive {
                Act::Dead
            } else if (x, z) != (was_x, was_z) {
                Act::Walk
            } else if aim.is_some() {
                Act::Attack
            } else if act == Act::Walk {
                Act::Idle
            } else {
                act
            };
            // Face the target (the model looks down +Z); the fallen keep their facing.
            let yaw = match aim {
                Some((ax, az)) if alive => Some(exact_game::math::atan2(ax - px, az - pz)),
                _ => None,
            };
            if i >= drawn.len() {
                let mut t = Transform::at(px, 0.2, pz).with_scale(SPEARMAN_SCALE);
                if let Some(yaw) = yaw {
                    t.rotation = Quat::from_rotation_y(yaw);
                }
                drawn.push(w.spawn((t, Mesh::asset(SPEARMAN_MODEL), clip(now))));
            } else {
                let e = drawn[i];
                let mut t = w.get_mut::<Transform>(e).unwrap();
                t.position = Vec3::new(px, 0.2, pz);
                if let Some(yaw) = yaw {
                    t.rotation = Quat::from_rotation_y(yaw);
                }
                drop(t);
                if now != act {
                    w.insert(e, clip(now));
                }
            }
            let entry = (now, x, z);
            if i < acts.len() {
                acts[i] = entry;
            } else {
                acts.push(entry);
            }
            continue;
        }
        let size = if troop == Troop::Brute { 0.95 } else { 0.6 };
        let pos = Vec3::new(px, if alive { size + 0.05 } else { -3.0 }, pz);
        if i >= drawn.len() {
            let e = w.spawn((
                Transform::at(pos.x, pos.y, pos.z),
                Mesh::capsule(size * 0.6, size * 1.2),
                Material::hex(troop_colour(troop)),
            ));
            drawn.push(e);
        } else {
            w.get_mut::<Transform>(drawn[i]).unwrap().position = pos;
        }
        if i >= acts.len() {
            acts.push((Act::Idle, x, z));
        }
    }
    let mut f = w.resource_mut::<Fight>();
    f.units = drawn;
    f.acts = acts;
}

/// Bar sizes in world units: a building's bar is most of its width, a troop's
/// about a tile; both thick enough to read on a phone.
const BAR_H_BUILDING: f32 = 0.5;
const BAR_H_TROOP: f32 = 0.34;
const BAR_W_TROOP: f32 = 1.2;
const BAR_EDGE: f32 = 0.08;

/// Where troop `i` is drawn, in world units (its spot plus the small per-index
/// nudge `draw_units` uses).
fn unit_spot(i: usize, x: i64, z: i64) -> (f32, f32) {
    let half = GRID as f32 / 2.0;
    let (ox, oz) = ((i * 7 % 5) as f32 * 0.12 - 0.24, (i * 3 % 7) as f32 * 0.08 - 0.24);
    (x as f32 / MILLI as f32 - half + ox, z as f32 / MILLI as f32 - half + oz)
}

fn world_xz(x: i64, z: i64) -> (f32, f32) {
    let half = GRID as f32 / 2.0;
    (x as f32 / MILLI as f32 - half, z as f32 / MILLI as f32 - half)
}

/// What wants a bar now: (is a troop, index, centre top in world, width,
/// height, fill 0..1). Only the damaged and still standing.
pub fn wanted_bars(b: &Battle) -> Vec<(bool, u32, Vec3, f32, f32, f32)> {
    let mut out = Vec::new();
    for (i, t) in b.targets.iter().enumerate() {
        if t.alive() && t.hp < t.max_hp {
            let (cx, cz) = t.centre();
            let (x, z) = world_xz(cx, cz);
            let s = t.kind.size() as f32;
            let top = 1.2 + s * 0.6;
            out.push((false, i as u32, Vec3::new(x, top, z), s * 0.75, BAR_H_BUILDING, t.hp as f32 / t.max_hp as f32));
        }
    }
    for (i, u) in b.units.iter().enumerate() {
        let (max, ..) = troop_stats(u.troop, b.troop_level);
        if u.alive() && u.hp < max {
            let (x, z) = unit_spot(i, u.x, u.z);
            let top = match u.troop {
                // Above the big-headed Knight's helmet at the steep camera.
                Troop::Brute => 3.0,
                Troop::Fighter => 2.9,
                Troop::Archer => 2.4,
            };
            out.push((true, i as u32, Vec3::new(x, top, z), BAR_W_TROOP, BAR_H_TROOP, u.hp as f32 / max as f32));
        }
    }
    out
}

/// A matte, glow-free colour: emission and shine wash a small bar out to
/// pastel under the village's bright light (seen in the 390×844 shots).
fn matte(css: &str) -> Material {
    Material { roughness: 1.0, ..Material::hex(css) }
}

fn bar_fill_look(unit: bool) -> Material {
    if unit {
        // The player's troops: green.
        matte("#00e04a")
    } else {
        // The enemy's buildings: red.
        matte("#e01010")
    }
}

/// HP bars and shots, drawn from the battle as it stands; nothing here feeds
/// back into the battle. Bars face the camera (they take its rotation).
pub fn draw_fx(w: &mut World, facing: Quat) {
    let (wanted, new_shots, tick, over) = match &w.resource::<Fight>().battle {
        Some(b) => (wanted_bars(b), b.shots.clone(), b.tick, b.over || w.resource::<Fight>().result.is_some()),
        None => (Vec::new(), Vec::new(), 0, true),
    };
    let wanted = if over { Vec::new() } else { wanted };
    let mut bars = std::mem::take(&mut w.resource_mut::<Fight>().bars);
    let mut flying = std::mem::take(&mut w.resource_mut::<Fight>().flying);

    // Bars: drop those no longer wanted, add new ones, then place them all.
    bars.retain(|bar| {
        let keep = wanted.iter().any(|&(unit, index, ..)| unit == bar.unit && index == bar.index);
        if !keep {
            w.despawn(bar.back);
            w.despawn(bar.fill);
        }
        keep
    });
    let right = facing * Vec3::X;
    let toward = facing * Vec3::Z;
    for &(unit, index, top, width, height, frac) in &wanted {
        let pos = bars.iter().position(|bar| bar.unit == unit && bar.index == index);
        let bar = match pos {
            Some(p) => bars[p].clone(),
            None => {
                let back = w.spawn((
                    Transform::at(top.x, top.y, top.z),
                    Mesh::cuboid(Vec3::new(width, height, 0.04)),
                    matte("#000000"),
                ));
                let inner = Vec3::new(width - 2.0 * BAR_EDGE, height - 2.0 * BAR_EDGE, 0.04);
                let fill = w.spawn((Transform::at(top.x, top.y, top.z), Mesh::cuboid(inner), bar_fill_look(unit)));
                let bar = Bar { unit, index, back, fill };
                bars.push(bar.clone());
                bar
            }
        };
        let frac = frac.clamp(0.01, 1.0);
        let inner_w = width - 2.0 * BAR_EDGE;
        {
            let mut t = w.get_mut::<Transform>(bar.back).unwrap();
            t.position = top;
            t.rotation = facing;
        }
        let mut t = w.get_mut::<Transform>(bar.fill).unwrap();
        // Left-anchored: the fill shrinks toward the bar's left end.
        t.position = top - right * (inner_w * (1.0 - frac) / 2.0) + toward * 0.03;
        t.rotation = facing;
        t.scale = Vec3::new(frac, 1.0, 1.0);
    }

    // Shots: each ranged hit of a new battle tick takes off now.
    let fresh = !over && tick != w.resource::<Fight>().shots_tick;
    let mut drawn = 0;
    if fresh {
        for s in &new_shots {
            let (fx, fz) = world_xz(s.from_x, s.from_z);
            let (tx, tz) = world_xz(s.to_x, s.to_z);
            // Leaves the cannon's mouth or the bow above the model, lands on the target.
            let from = Vec3::new(fx, if s.ball { 1.6 } else { 1.3 }, fz);
            let to = Vec3::new(tx, if s.ball { 0.8 } else { 1.0 }, tz);
            // Arrows fly 14 tiles a second, cannonballs 9.
            let speed = if s.ball { 9.0 } else { 14.0 };
            // At least a third of a second in the air, so even a point-blank
            // shot is seen.
            let ticks = ((from.distance(to) / speed * TPS as f32) as u32).max(10);
            let yaw = exact_game::math::atan2(to.x - from.x, to.z - from.z);
            let entity = if s.ball {
                w.spawn((
                    Transform::at(from.x, from.y, from.z),
                    Mesh::sphere(0.34),
                    Material::hex("#1a1a1a").emissive(1.0, 0.45, 0.05),
                ))
            } else {
                let mut t = Transform::at(from.x, from.y, from.z);
                t.rotation = Quat::from_rotation_y(yaw);
                w.spawn((t, Mesh::cuboid(Vec3::new(0.14, 0.14, 0.8)), Material::hex("#fff59d").emissive(0.9, 0.85, 0.35)))
            };
            flying.push(Flying { entity, from, to, born: tick, ticks, ball: s.ball });
            drawn += 1;
        }
    }
    flying.retain(|fly| {
        let age = tick.saturating_sub(fly.born);
        let keep = !over && age <= fly.ticks;
        if !keep {
            w.despawn(fly.entity);
        }
        keep
    });
    for fly in &flying {
        let k = (tick.saturating_sub(fly.born) as f32 / fly.ticks as f32).clamp(0.0, 1.0);
        let lift = if fly.ball { 1.8 } else { 0.5 } * 4.0 * k * (1.0 - k);
        let at = fly.from.lerp(fly.to, k) + Vec3::Y * lift;
        w.get_mut::<Transform>(fly.entity).unwrap().position = at;
    }

    let mut f = w.resource_mut::<Fight>();
    f.bars = bars;
    f.flying = flying;
    if fresh {
        f.shots_tick = tick;
        f.shots_drawn += drawn;
    }
}
