//! Battles against computer camps. Pure rules with whole-number maths, so the
//! same drops always give the same fight and tests can replay it. Positions
//! are in thousandths of a tile; time is in ticks of the village's 30 Hz.

use crate::economy::{Kind, Town, Troop, GRID, TROOPS};
use exact_game::*;

/// Ticks per second (the village's HZ).
pub const TPS: u32 = 30;
/// A battle lasts two and a half minutes.
pub const BATTLE_TICKS: u32 = 150 * TPS;
/// A thousandth of a tile.
pub const MILLI: i64 = 1000;

/// One building in a camp's layout: kind, level, top-left tile, loot held.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub kind: Kind,
    pub level: u8,
    pub x: i32,
    pub z: i32,
    pub gold: u32,
    pub elixir: u32,
}

const fn p(kind: Kind, level: u8, x: i32, z: i32, gold: u32, elixir: u32) -> Plan {
    Plan { kind, level, x, z, gold, elixir }
}

/// The campaign: four rival strongholds to raid, easiest first. A camp is the
/// same list of placed buildings a player's village becomes (`plans_from_town`),
/// so raiding another player later is the same battle.
pub const CAMPS: [(&str, &[Plan]); 4] = [
    (
        "Bandit Camp",
        &[
            p(Kind::TownHall, 1, 13, 13, 100, 100),
            p(Kind::GoldMine, 1, 8, 13, 150, 0),
            p(Kind::ElixirCollector, 1, 19, 13, 0, 150),
            p(Kind::GoldStorage, 1, 13, 8, 200, 0),
            p(Kind::ElixirStorage, 1, 13, 19, 0, 200),
        ],
    ),
    (
        "Raider Outpost",
        &[
            p(Kind::TownHall, 1, 13, 13, 150, 150),
            p(Kind::Cannon, 1, 13, 9, 0, 0),
            p(Kind::GoldMine, 1, 8, 12, 200, 0),
            p(Kind::ElixirCollector, 1, 19, 12, 0, 200),
            p(Kind::GoldStorage, 1, 9, 17, 300, 0),
            p(Kind::ElixirStorage, 1, 18, 17, 0, 300),
        ],
    ),
    (
        "Rebel Fort",
        &[
            p(Kind::TownHall, 2, 13, 13, 200, 200),
            p(Kind::Cannon, 1, 9, 9, 0, 0),
            p(Kind::Cannon, 1, 18, 18, 0, 0),
            p(Kind::ArcherTower, 1, 18, 9, 0, 0),
            p(Kind::GoldStorage, 2, 9, 18, 500, 0),
            p(Kind::ElixirStorage, 2, 13, 18, 0, 500),
            p(Kind::GoldMine, 2, 5, 13, 250, 0),
            p(Kind::ElixirCollector, 2, 22, 13, 0, 250),
        ],
    ),
    (
        "Warlord's Citadel",
        &[
            p(Kind::TownHall, 3, 13, 13, 300, 300),
            p(Kind::Cannon, 2, 9, 9, 0, 0),
            p(Kind::Cannon, 2, 18, 9, 0, 0),
            p(Kind::ArcherTower, 2, 9, 18, 0, 0),
            p(Kind::ArcherTower, 2, 18, 18, 0, 0),
            p(Kind::GoldStorage, 3, 13, 9, 800, 0),
            p(Kind::ElixirStorage, 3, 13, 18, 0, 800),
            p(Kind::GoldMine, 2, 5, 13, 300, 0),
            p(Kind::ElixirCollector, 2, 22, 13, 0, 300),
        ],
    ),
];

/// Hit points a camp building starts with.
pub fn building_hp(kind: Kind) -> i32 {
    match kind {
        Kind::TownHall => 400,
        Kind::Cannon | Kind::ArcherTower => 300,
        Kind::GoldStorage | Kind::ElixirStorage => 250,
        Kind::GoldMine | Kind::ElixirCollector => 200,
        _ => 150,
    }
}

/// A defense's (range in milli-tiles, damage per shot, ticks between shots).
pub fn defense(kind: Kind, level: u8) -> Option<(i64, i32, u32)> {
    let i = usize::from(level.clamp(1, 8) - 1);
    match kind {
        // Every 0.9 s and every 0.45 s.
        Kind::Cannon => Some((7 * MILLI + 500, [10, 14, 18, 23, 29, 36, 44, 53][i], 27)),
        Kind::ArcherTower => Some((8 * MILLI + 500, [5, 7, 9, 12, 15, 19, 23, 28][i], 14)),
        _ => None,
    }
}

/// A troop's (hit points, damage per hit, ticks between hits, milli-tiles per tick, reach in milli-tiles).
pub fn stats(t: Troop) -> (i32, i32, u32, i64, i64) {
    match t {
        // 9 damage a second, 1.5 tiles a second.
        Troop::Fighter => (55, 9, TPS, 50, 700),
        Troop::Archer => (24, 8, TPS, 50, 4000),
        // 12 damage a second, 0.9 tiles a second.
        Troop::Brute => (360, 12, TPS, 30, 700),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Target {
    pub kind: Kind,
    pub level: u8,
    pub x: i32,
    pub z: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub gold: u32,
    pub elixir: u32,
    /// Looted on an earlier raid: gives nothing this time.
    pub spent: bool,
    pub cooldown: u32,
}

impl Target {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
    /// The footprint's centre in milli-tiles.
    pub fn centre(&self) -> (i64, i64) {
        let half = i64::from(self.kind.size()) * MILLI / 2;
        (i64::from(self.x) * MILLI + half, i64::from(self.z) * MILLI + half)
    }
    /// Half the footprint, in milli-tiles: troops reach the edge, not the middle.
    pub fn half(&self) -> i64 {
        i64::from(self.kind.size()) * MILLI / 2
    }
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Unit {
    pub troop: Troop,
    pub x: i64,
    pub z: i64,
    pub hp: i32,
    /// Index into the battle's targets.
    pub target: Option<u32>,
    pub cooldown: u32,
}

impl Unit {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Time,
    Wiped,
    Destroyed,
    Surrender,
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Battle {
    pub camp: u32,
    pub targets: Vec<Target>,
    pub units: Vec<Unit>,
    pub tick: u32,
    /// Troops still to drop, per `TROOPS` index.
    pub reserve: Vec<u32>,
    pub gold: u32,
    pub elixir: u32,
    pub over: bool,
    pub deployed: u32,
}

/// Distance squared, in milli-tiles squared.
fn d2(ax: i64, az: i64, bx: i64, bz: i64) -> i64 {
    (ax - bx) * (ax - bx) + (az - bz) * (az - bz)
}

impl Battle {
    /// A battle at camp `camp` with `reserve` troops to drop. `looted` marks
    /// buildings (by index in the camp plan) already looted on earlier raids.
    pub fn new(camp: usize, reserve: Vec<u32>, looted: u32) -> Battle {
        let plan = CAMPS[camp.min(CAMPS.len() - 1)].1;
        Battle { camp: camp as u32, ..Battle::on(plan, reserve, looted) }
    }

    /// A battle against any layout (a campaign camp or a player's village).
    pub fn on(plan: &[Plan], reserve: Vec<u32>, looted: u32) -> Battle {
        let targets = plan
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let hp = building_hp(b.kind);
                Target {
                    kind: b.kind,
                    level: b.level,
                    x: b.x,
                    z: b.z,
                    hp,
                    max_hp: hp,
                    gold: b.gold,
                    elixir: b.elixir,
                    spent: looted & (1 << i) != 0,
                    cooldown: 0,
                }
            })
            .collect();
        Battle { targets, reserve, ..Battle::default() }
    }

    pub fn ticks_left(&self) -> u32 {
        BATTLE_TICKS.saturating_sub(self.tick)
    }
    pub fn destroyed(&self) -> usize {
        self.targets.iter().filter(|t| !t.alive()).count()
    }
    /// Destruction, 0 to 100, by buildings destroyed (the Town Hall counts).
    pub fn percent(&self) -> u32 {
        (self.destroyed() * 100 / self.targets.len().max(1)) as u32
    }
    pub fn town_hall_down(&self) -> bool {
        self.targets.iter().any(|t| t.kind == Kind::TownHall && !t.alive())
    }
    pub fn stars(&self) -> u8 {
        let pct = self.percent();
        u8::from(pct >= 50) + u8::from(self.town_hall_down()) + u8::from(pct == 100)
    }
    /// Buildings looted in this battle, as a bitmask by plan index.
    pub fn looted_now(&self) -> u32 {
        self.targets
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.alive() && !t.spent)
            .fold(0, |m, (i, _)| m | (1 << i))
    }

    /// Why a troop can't be dropped on tile (x, z): off the map or inside the
    /// red border (one tile around any building).
    pub fn can_deploy(&self, t: Troop, x: i32, z: i32) -> Result<(), String> {
        if self.over {
            return Err("The battle is over".into());
        }
        if !(0..GRID).contains(&x) || !(0..GRID).contains(&z) {
            return Err("Off the map".into());
        }
        if self.reserve.get(t.index()).copied().unwrap_or(0) == 0 {
            return Err(format!("No {} left", t.plural()));
        }
        let blocked = self.targets.iter().any(|b| {
            let s = b.kind.size();
            x >= b.x - 1 && x < b.x + s + 1 && z >= b.z - 1 && z < b.z + s + 1
        });
        if blocked {
            return Err("Too close: drop outside the red border".into());
        }
        Ok(())
    }

    /// Drops one troop on tile (x, z). Each drop lands a little apart from
    /// the last so a crowd doesn't walk as one blob.
    pub fn deploy(&mut self, t: Troop, x: i32, z: i32) -> Result<(), String> {
        self.can_deploy(t, x, z)?;
        self.reserve[t.index()] -= 1;
        let n = i64::from(self.deployed);
        let (dx, dz) = ((n % 5 - 2) * 150, (n / 5 % 5 - 2) * 150);
        let (hp, ..) = stats(t);
        self.units.push(Unit {
            troop: t,
            x: i64::from(x) * MILLI + MILLI / 2 + dx,
            z: i64::from(z) * MILLI + MILLI / 2 + dz,
            hp,
            target: None,
            cooldown: 0,
        });
        self.deployed += 1;
        Ok(())
    }

    /// The nearest living building this troop wants.
    fn pick(&self, u: &Unit) -> Option<usize> {
        let wants_defense = u.troop == Troop::Brute
            && self.targets.iter().any(|t| t.alive() && t.kind.is_defense());
        self.targets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.alive() && (!wants_defense || t.kind.is_defense()))
            .min_by_key(|(i, t)| {
                let (cx, cz) = t.centre();
                (d2(u.x, u.z, cx, cz), *i)
            })
            .map(|(i, _)| i)
    }

    /// One tick: troops pick, walk and hit; defenses shoot; then check the end.
    pub fn step(&mut self) -> Option<End> {
        if self.over {
            return None;
        }
        self.tick += 1;
        // Troops.
        for i in 0..self.units.len() {
            if !self.units[i].alive() {
                continue;
            }
            let still = self.units[i].target.map(|t| t as usize).filter(|&t| self.targets[t].alive());
            let target = match still {
                Some(t) => Some(t),
                None => self.pick(&self.units[i]),
            };
            self.units[i].target = target.map(|t| t as u32);
            let Some(ti) = target else { continue };
            let (_, damage, every, speed, reach) = stats(self.units[i].troop);
            let (cx, cz) = self.targets[ti].centre();
            let reach = reach + self.targets[ti].half();
            let u = &mut self.units[i];
            let dist = d2(u.x, u.z, cx, cz).isqrt().max(1);
            if dist > reach {
                let step = speed.min(dist - reach);
                let (mx, mz) = ((cx - u.x) * step / dist, (cz - u.z) * step / dist);
                // Whole-number rounding can leave the last sliver as no move at
                // all; then it is close enough to swing.
                if mx != 0 || mz != 0 {
                    u.x += mx;
                    u.z += mz;
                    u.cooldown = u.cooldown.saturating_sub(1);
                    continue;
                }
            }
            if u.cooldown > 0 {
                u.cooldown -= 1;
                continue;
            }
            u.cooldown = every;
            let b = &mut self.targets[ti];
            b.hp = (b.hp - damage).max(0);
            if !b.alive() && !b.spent {
                self.gold += b.gold;
                self.elixir += b.elixir;
            }
        }
        // Defenses.
        for bi in 0..self.targets.len() {
            let b = &self.targets[bi];
            if !b.alive() {
                continue;
            }
            let Some((range, damage, every)) = defense(b.kind, b.level) else { continue };
            if b.cooldown > 0 {
                self.targets[bi].cooldown -= 1;
                continue;
            }
            let (cx, cz) = b.centre();
            let reach = range + b.half();
            let aim = self
                .units
                .iter()
                .enumerate()
                .filter(|(_, u)| u.alive() && d2(u.x, u.z, cx, cz) <= reach * reach)
                .min_by_key(|(i, u)| (d2(u.x, u.z, cx, cz), *i))
                .map(|(i, _)| i);
            if let Some(ui) = aim {
                let u = &mut self.units[ui];
                u.hp = (u.hp - damage).max(0);
                self.targets[bi].cooldown = every;
            }
        }
        self.check_end()
    }

    fn check_end(&mut self) -> Option<End> {
        let end = if self.destroyed() == self.targets.len() {
            Some(End::Destroyed)
        } else if self.tick >= BATTLE_TICKS {
            Some(End::Time)
        } else if self.deployed > 0
            && self.units.iter().all(|u| !u.alive())
            && self.reserve.iter().all(|&n| n == 0)
        {
            Some(End::Wiped)
        } else {
            None
        };
        if end.is_some() {
            self.over = true;
        }
        end
    }

    pub fn surrender(&mut self) {
        self.over = true;
    }

    /// Troops of each kind still standing (they don't come home: used up).
    pub fn alive(&self, t: Troop) -> u32 {
        self.units.iter().filter(|u| u.troop == t && u.alive()).count() as u32
    }
}

/// A player's village as a camp to raid: every finished building where it
/// stands. A fifth of what sits in storage (shared across its storages) and
/// what waits in each mine or collector is up for grabs.
pub fn plans_from_town(town: &Town) -> Vec<Plan> {
    let gold_stores = town.buildings.iter().filter(|b| b.kind == Kind::GoldStorage && b.level > 0).count() as u32;
    let elixir_stores = town.buildings.iter().filter(|b| b.kind == Kind::ElixirStorage && b.level > 0).count() as u32;
    town.buildings
        .iter()
        .filter(|b| b.level > 0)
        .map(|b| {
            let (gold, elixir) = match b.kind {
                Kind::GoldStorage => (town.gold / 5 / gold_stores.max(1), 0),
                Kind::ElixirStorage => (0, town.elixir / 5 / elixir_stores.max(1)),
                Kind::GoldMine => ((b.banked_milli / 1000) as u32 / 2, 0),
                Kind::ElixirCollector => (0, (b.banked_milli / 1000) as u32 / 2),
                _ => (0, 0),
            };
            Plan { kind: b.kind, level: b.level, x: b.x, z: b.z, gold, elixir }
        })
        .collect()
}

/// Sanity: the battle knows every troop kind the army has.
const _: () = assert!(TROOPS.len() == 3 && CAMPS.len() == crate::economy::CAMP_COUNT);
