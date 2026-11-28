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
