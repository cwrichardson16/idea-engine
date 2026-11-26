//! The village's rules: what buildings cost, how long they take, what they
//! produce and hold, and how builders are shared. No graphics here: every
//! function takes "now" in village milliseconds, so the same rules run while
//! playing, after time away, and in tests.

use exact_game::*;

/// The village is GRID × GRID tiles.
pub const GRID: i32 = 30;
/// The highest level of any building in this slice.
pub const MAX_LEVEL: u8 = 3;

pub const START_GOLD: u32 = 500;
pub const START_ELIXIR: u32 = 500;
pub const START_GEMS: u32 = 50;
/// Speed-ups: one gem finishes this many seconds of a timer (rounded up).
pub const GEM_SECONDS: u64 = 10;
/// Speed-ups: one Knowledge finishes this many seconds (study pays better than gems).
pub const KNOWLEDGE_SECONDS: u64 = 20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Data)]
pub enum Kind {
    #[default]
    TownHall,
    GoldMine,
    ElixirCollector,
    GoldStorage,
    ElixirStorage,
    BuilderHut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res {
    Gold,
    Elixir,
}

pub const KINDS: [Kind; 6] = [
    Kind::TownHall,
    Kind::GoldMine,
    Kind::ElixirCollector,
    Kind::GoldStorage,
    Kind::ElixirStorage,
    Kind::BuilderHut,
];

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::TownHall => "townhall",
            Kind::GoldMine => "goldmine",
            Kind::ElixirCollector => "collector",
            Kind::GoldStorage => "goldstorage",
            Kind::ElixirStorage => "elixirstorage",
            Kind::BuilderHut => "builderhut",
        }
    }
    pub fn from_key(key: &str) -> Option<Kind> {
        KINDS.into_iter().find(|k| k.key() == key)
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::TownHall => "Town Hall",
            Kind::GoldMine => "Gold Mine",
            Kind::ElixirCollector => "Elixir Collector",
            Kind::GoldStorage => "Gold Storage",
            Kind::ElixirStorage => "Elixir Storage",
            Kind::BuilderHut => "Builder Hut",
        }
    }
    /// Footprint, in tiles per side.
    pub fn size(self) -> i32 {
        match self {
            Kind::TownHall => 4,
            Kind::BuilderHut => 2,
            _ => 3,
        }
    }
    pub fn max_level(self) -> u8 {
        match self {
            Kind::BuilderHut => 1,
            _ => MAX_LEVEL,
        }
    }
    /// What reaching `level` costs (level 1 is placing it).
    pub fn cost(self, level: u8) -> (Res, u32) {
        let i = usize::from(level.clamp(1, MAX_LEVEL) - 1);
        match self {
            Kind::TownHall => (Res::Gold, [0, 1000, 3000][i]),
            Kind::GoldMine => (Res::Elixir, [150, 300, 700][i]),
            Kind::ElixirCollector => (Res::Gold, [150, 300, 700][i]),
            Kind::GoldStorage => (Res::Elixir, [300, 600, 1200][i]),
            Kind::ElixirStorage => (Res::Gold, [300, 600, 1200][i]),
            Kind::BuilderHut => (Res::Gold, [500, 0, 0][i]),
        }
    }
    /// Seconds a builder spends reaching `level`. Zero is instant and needs no builder.
    pub fn build_secs(self, level: u8) -> u64 {
        let i = usize::from(level.clamp(1, MAX_LEVEL) - 1);
        match self {
            Kind::TownHall => [0, 60, 120][i],
            Kind::GoldMine | Kind::ElixirCollector => [10, 30, 60][i],
            Kind::GoldStorage | Kind::ElixirStorage => [15, 40, 80][i],
            Kind::BuilderHut => 0,
        }
    }
    /// What a mine or collector makes: (resource, per second, holds).
    pub fn production(self, level: u8) -> Option<(Res, u64, u64)> {
        if level == 0 {
            return None;
        }
        let i = usize::from(level.min(MAX_LEVEL) - 1);
        let rate = [2, 3, 4][i];
        let holds = [100, 200, 300][i];
        match self {
            Kind::GoldMine => Some((Res::Gold, rate, holds)),
            Kind::ElixirCollector => Some((Res::Elixir, rate, holds)),
            _ => None,
        }
    }
    /// How much of each resource it lets the village hold: (gold, elixir).
    pub fn holds(self, level: u8) -> (u32, u32) {
        if level == 0 {
            return (0, 0);
        }
        let i = usize::from(level.min(MAX_LEVEL) - 1);
        match self {
            Kind::TownHall => {
                let n = [1000, 2000, 3000][i];
                (n, n)
            }
            Kind::GoldStorage => ([1500, 3000, 5000][i], 0),
            Kind::ElixirStorage => (0, [1500, 3000, 5000][i]),
            _ => (0, 0),
        }
    }
    /// How many the Town Hall at `th` level allows.
    pub fn max_count(self, th: u8) -> usize {
        let i = usize::from(th.clamp(1, MAX_LEVEL) - 1);
        match self {
            Kind::TownHall => 1,
            Kind::GoldMine | Kind::ElixirCollector => [1, 2, 3][i],
            Kind::GoldStorage | Kind::ElixirStorage => [1, 1, 2][i],
            Kind::BuilderHut => 1,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Data)]
pub struct Building {
    pub id: u32,
    pub kind: Kind,
    /// Finished level; 0 while it is first being built.
    pub level: u8,
    /// Top-left tile.
    pub x: i32,
    pub z: i32,
    /// When its builder finishes, in village ms; 0 when no one is working on it.
    pub done_at: u64,
    /// When that work started, for progress bars.
    pub started_at: u64,
    /// Production in thousandths of a unit, banked at `mark`.
    pub banked_milli: u64,
    /// The village time production last counted from.
    pub mark: u64,
}

impl Building {
    pub fn busy(&self) -> bool {
        self.done_at != 0
    }
    pub fn covers(&self, x: i32, z: i32) -> bool {
        let s = self.kind.size();
        x >= self.x && x < self.x + s && z >= self.z && z < self.z + s
    }
    /// Thousandths of a unit waiting to be collected at `now`. Nothing is
    /// made while it is being built or upgraded.
    pub fn waiting_milli(&self, now: u64) -> u64 {
        let Some((_, rate, holds)) = self.kind.production(self.level) else {
            return 0;
        };
        let made = if self.busy() {
            0
        } else {
            rate * now.saturating_sub(self.mark)
        };
        (self.banked_milli + made).min(holds * 1000)
    }
    /// Whole units waiting at `now`.
    pub fn waiting(&self, now: u64) -> u32 {
        (self.waiting_milli(now) / 1000) as u32
    }
}

#[derive(Default, Resource)]
pub struct Town {
    pub gold: u32,
    pub elixir: u32,
    pub gems: u32,
    pub knowledge: u32,
    pub buildings: Vec<Building>,
    pub next_id: u32,
}

/// How a timer gets skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pay {
    Gems,
    Knowledge,
}

impl Town {
    /// A new village: a level-1 Town Hall in the middle and starting resources.
    pub fn new() -> Town {
        let s = Kind::TownHall.size();
        let mid = (GRID - s) / 2;
        Town {
            gold: START_GOLD,
            elixir: START_ELIXIR,
            gems: START_GEMS,
            knowledge: 0,
            buildings: vec![Building {
                id: 1,
                kind: Kind::TownHall,
                level: 1,
                x: mid,
                z: mid,
                ..Building::default()
            }],
            next_id: 2,
        }
    }

    pub fn get(&self, id: u32) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == id)
    }
    fn index(&self, id: u32) -> Option<usize> {
        self.buildings.iter().position(|b| b.id == id)
    }
    pub fn at(&self, x: i32, z: i32) -> Option<&Building> {
        self.buildings.iter().find(|b| b.covers(x, z))
    }
    pub fn town_hall_level(&self) -> u8 {
        self.buildings
            .iter()
            .find(|b| b.kind == Kind::TownHall)
            .map_or(1, |b| b.level.max(1))
    }
    pub fn count(&self, kind: Kind) -> usize {
        self.buildings.iter().filter(|b| b.kind == kind).count()
    }
    /// (gold, elixir) the village can hold, from finished levels.
    pub fn caps(&self) -> (u32, u32) {
        self.buildings.iter().fold((0, 0), |(g, e), b| {
            let (bg, be) = b.kind.holds(b.level);
            (g + bg, e + be)
        })
    }
    pub fn builders(&self) -> usize {
        1 + self
            .buildings
            .iter()
            .filter(|b| b.kind == Kind::BuilderHut && b.level >= 1)
            .count()
    }
    pub fn builders_busy(&self) -> usize {
        self.buildings.iter().filter(|b| b.busy()).count()
    }
    pub fn builders_free(&self) -> usize {
        self.builders().saturating_sub(self.builders_busy())
    }
    pub fn amount(&self, r: Res) -> u32 {
        match r {
            Res::Gold => self.gold,
            Res::Elixir => self.elixir,
        }
    }
    fn spend(&mut self, r: Res, n: u32) {
        match r {
            Res::Gold => self.gold -= n,
            Res::Elixir => self.elixir -= n,
        }
    }
    /// Adds what fits under the caps and returns how much fitted.
    fn earn(&mut self, r: Res, n: u32) -> u32 {
        let (gc, ec) = self.caps();
        let (have, cap) = match r {
            Res::Gold => (&mut self.gold, gc),
            Res::Elixir => (&mut self.elixir, ec),
        };
        let fits = n.min(cap.saturating_sub(*have));
        *have += fits;
        fits
    }

    fn afford(&self, r: Res, n: u32) -> Result<(), String> {
        if self.amount(r) < n {
            let what = match r {
                Res::Gold => "gold",
                Res::Elixir => "elixir",
            };
            return Err(format!("Need {n} {what}"));
        }
        Ok(())
    }

    /// Why `kind` can't be bought now, regardless of where it goes.
    pub fn can_buy(&self, kind: Kind) -> Result<(), String> {
        let th = self.town_hall_level();
        if self.count(kind) >= kind.max_count(th) {
            return Err(if self.count(kind) >= kind.max_count(MAX_LEVEL) {
                "Max built".into()
            } else {
                "Upgrade Town Hall".into()
            });
        }
        let (r, n) = kind.cost(1);
        self.afford(r, n)?;
        if kind.build_secs(1) > 0 && self.builders_free() == 0 {
            return Err("No free builder".into());
        }
        Ok(())
    }

    /// Why `kind` can't go with its top-left at tile (x, z).
    pub fn can_place(&self, kind: Kind, x: i32, z: i32) -> Result<(), String> {
        self.can_buy(kind)?;
        let s = kind.size();
        if x < 0 || z < 0 || x + s > GRID || z + s > GRID {
            return Err("Off the edge".into());
        }
        let overlaps = self.buildings.iter().any(|b| {
            let bs = b.kind.size();
            x < b.x + bs && b.x < x + s && z < b.z + bs && b.z < z + s
        });
        if overlaps {
            return Err("Something is in the way".into());
        }
        Ok(())
    }

    /// Buys and places `kind`; a builder starts on it unless it's instant.
    pub fn place(&mut self, kind: Kind, x: i32, z: i32, now: u64) -> Result<u32, String> {
        self.settle(now);
        self.can_place(kind, x, z)?;
        let (r, n) = kind.cost(1);
        self.spend(r, n);
        let id = self.next_id;
        self.next_id += 1;
        let secs = kind.build_secs(1);
        let mut b = Building {
            id,
            kind,
            x,
            z,
            mark: now,
            ..Building::default()
        };
        if secs == 0 {
            b.level = 1;
        } else {
            b.started_at = now;
            b.done_at = now + secs * 1000;
        }
        self.buildings.push(b);
        Ok(id)
    }

    /// Why building `id` can't be upgraded now.
    pub fn can_upgrade(&self, id: u32) -> Result<(), String> {
        let b = self.get(id).ok_or("No such building")?;
        if b.busy() {
            return Err("Already being built".into());
        }
        if b.level >= b.kind.max_level() {
            return Err("Max level".into());
        }
        let next = b.level + 1;
        if b.kind != Kind::TownHall && next > self.town_hall_level() {
            return Err("Upgrade Town Hall first".into());
        }
        let (r, n) = b.kind.cost(next);
        self.afford(r, n)?;
        if self.builders_free() == 0 {
            return Err("No free builder".into());
        }
        Ok(())
    }

    pub fn upgrade(&mut self, id: u32, now: u64) -> Result<(), String> {
        self.settle(now);
        self.can_upgrade(id)?;
        let i = self.index(id).ok_or("No such building")?;
        let next = self.buildings[i].level + 1;
        let (r, n) = self.buildings[i].kind.cost(next);
        self.spend(r, n);
        let secs = self.buildings[i].kind.build_secs(next);
        let b = &mut self.buildings[i];
        // Bank what it made so far; it pauses while upgrading.
        b.banked_milli = b.waiting_milli(now);
        b.mark = now;
        b.started_at = now;
        b.done_at = now + secs * 1000;
        Ok(())
    }

    /// Finishes every timer due by `now`, in the order they end. Returns how many finished.
    pub fn settle(&mut self, now: u64) -> usize {
        let mut finished = 0;
        loop {
            let next = self
                .buildings
                .iter()
                .enumerate()
                .filter(|(_, b)| b.busy() && b.done_at <= now)
                .min_by_key(|(_, b)| (b.done_at, b.id))
                .map(|(i, _)| i);
            let Some(i) = next else { break };
            let at = self.buildings[i].done_at;
            self.finish(i, at);
            finished += 1;
        }
        finished
    }

    /// Ends building `i`'s work at village time `at`: it gains its level and
    /// starts producing from that moment.
    fn finish(&mut self, i: usize, at: u64) {
        let b = &mut self.buildings[i];
        b.level += 1;
        b.mark = at;
        b.done_at = 0;
        b.started_at = 0;
    }

    /// Collects a mine or collector into storage; returns what moved. What
    /// doesn't fit stays in the building.
    pub fn collect(&mut self, id: u32, now: u64) -> u32 {
        self.settle(now);
        let Some(i) = self.index(id) else { return 0 };
        let b = &self.buildings[i];
        let Some((r, _, _)) = b.kind.production(b.level) else {
            return 0;
        };
        let have = b.waiting_milli(now);
        let busy = b.busy();
        let moved = self.earn(r, (have / 1000) as u32);
        let b = &mut self.buildings[i];
        b.banked_milli = have - u64::from(moved) * 1000;
        if !busy {
            b.mark = now;
        }
        moved
    }

    /// Collects every mine and collector.
    pub fn collect_all(&mut self, now: u64) -> u32 {
        let ids: Vec<u32> = self.buildings.iter().map(|b| b.id).collect();
        ids.into_iter().map(|id| self.collect(id, now)).sum()
    }

    /// What finishing `id`'s timer right now costs, or None if it has no timer.
    pub fn skip_cost(&self, id: u32, now: u64, pay: Pay) -> Option<u32> {
        let b = self.get(id)?;
        if !b.busy() || b.done_at <= now {
            return None;
        }
        let left_s = (b.done_at - now).div_ceil(1000);
        let per = match pay {
            Pay::Gems => GEM_SECONDS,
            Pay::Knowledge => KNOWLEDGE_SECONDS,
        };
        Some(left_s.div_ceil(per).max(1) as u32)
    }

    /// Finishes `id`'s timer now, paying with gems or Knowledge.
    pub fn skip(&mut self, id: u32, now: u64, pay: Pay) -> Result<(), String> {
        self.settle(now);
        let cost = self.skip_cost(id, now, pay).ok_or("Nothing to finish")?;
        let have = match pay {
            Pay::Gems => &mut self.gems,
            Pay::Knowledge => &mut self.knowledge,
        };
        if *have < cost {
            return Err(match pay {
                Pay::Gems => format!("Need {cost} gems"),
                Pay::Knowledge => format!("Need {cost} Knowledge"),
            });
        }
        *have -= cost;
        let i = self.index(id).ok_or("No such building")?;
        self.finish(i, now);
        Ok(())
    }

    /// The village as one line of text, for the page to keep between visits.
    pub fn encode(&self) -> String {
        let mut out = format!(
            "{},{},{},{},{}",
            self.gold, self.elixir, self.gems, self.knowledge, self.next_id
        );
        for b in &self.buildings {
            let kind = KINDS.iter().position(|k| *k == b.kind).unwrap_or(0);
            out += &format!(
                ";{},{},{},{},{},{},{},{},{}",
                b.id, kind, b.level, b.x, b.z, b.done_at, b.started_at, b.banked_milli, b.mark
            );
        }
        out
    }

    /// Reads `encode`'s text back; None if it isn't a village.
    pub fn decode(text: &str) -> Option<Town> {
        let mut parts = text.split(';');
        let head: Vec<u32> = parts
            .next()?
            .split(',')
            .map(|n| n.parse().ok())
            .collect::<Option<_>>()?;
        let [gold, elixir, gems, knowledge, next_id] = head[..] else {
            return None;
        };
        let mut buildings = Vec::new();
        for p in parts {
            let f: Vec<u64> = p.split(',').map(|n| n.parse().ok()).collect::<Option<_>>()?;
            let [id, kind, level, x, z, done_at, started_at, banked_milli, mark] = f[..] else {
                return None;
            };
            buildings.push(Building {
                id: id as u32,
                kind: *KINDS.get(kind as usize)?,
                level: u8::try_from(level).ok()?.min(MAX_LEVEL),
                x: i32::try_from(x).ok()?,
                z: i32::try_from(z).ok()?,
                done_at,
                started_at,
                banked_milli,
                mark,
            });
        }
        if !buildings.iter().any(|b| b.kind == Kind::TownHall) {
            return None;
        }
        Some(Town {
            gold,
            elixir,
            gems,
            knowledge,
            buildings,
            next_id,
        })
    }

    /// The study hook: one due review earns one Knowledge. (Anki sends these later.)
    pub fn review(&mut self) {
        self.knowledge += 1;
    }

    /// Where a building of `kind` would first fit, scanning from the middle out.
    pub fn free_spot(&self, kind: Kind) -> Option<(i32, i32)> {
        let s = kind.size();
        let mut spots: Vec<(i32, i32)> = (0..=GRID - s)
            .flat_map(|z| (0..=GRID - s).map(move |x| (x, z)))
            .collect();
        let mid = (GRID - s) / 2;
        spots.sort_by_key(|&(x, z)| ((x - mid).abs().max((z - mid).abs()), z, x));
        spots.into_iter().find(|&(x, z)| {
            !self.buildings.iter().any(|b| {
                let bs = b.kind.size();
                // Keep a one-tile path around everything.
                x < b.x + bs + 1 && b.x < x + s + 1 && z < b.z + bs + 1 && b.z < z + s + 1
            })
        })
    }
}
