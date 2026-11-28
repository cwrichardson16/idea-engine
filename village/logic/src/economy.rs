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
    // Slice 2. New kinds only ever go at the end: saves store a kind's position.
    Barracks,
    ArmyCamp,
    Cannon,
    ArcherTower,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res {
    Gold,
    Elixir,
}

pub const KINDS: [Kind; 10] = [
    Kind::TownHall,
    Kind::GoldMine,
    Kind::ElixirCollector,
    Kind::GoldStorage,
    Kind::ElixirStorage,
    Kind::BuilderHut,
    Kind::Barracks,
    Kind::ArmyCamp,
    Kind::Cannon,
    Kind::ArcherTower,
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
            Kind::Barracks => "barracks",
            Kind::ArmyCamp => "armycamp",
            Kind::Cannon => "cannon",
            Kind::ArcherTower => "archertower",
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
            Kind::Barracks => "Barracks",
            Kind::ArmyCamp => "Army Camp",
            Kind::Cannon => "Cannon",
            Kind::ArcherTower => "Archer Tower",
        }
    }
    /// Footprint, in tiles per side.
    pub fn size(self) -> i32 {
        match self {
            Kind::TownHall | Kind::ArmyCamp => 4,
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
            Kind::Barracks => (Res::Elixir, [180, 550, 1400][i]),
            Kind::ArmyCamp => (Res::Elixir, [220, 700, 1600][i]),
            Kind::Cannon => (Res::Gold, [300, 800, 1700][i]),
            Kind::ArcherTower => (Res::Gold, [900, 1800, 3200][i]),
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
            Kind::Barracks | Kind::ArmyCamp => [25, 70, 140][i],
            Kind::Cannon => [20, 50, 100][i],
            Kind::ArcherTower => [50, 110, 200][i],
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
            Kind::BuilderHut | Kind::Barracks => 1,
            Kind::ArmyCamp => [1, 1, 2][i],
            Kind::Cannon => [2, 2, 3][i],
            Kind::ArcherTower => [0, 1, 2][i],
        }
    }
    /// The Town Hall level that first allows one.
    pub fn unlocks_at(self) -> u8 {
        (1..=MAX_LEVEL).find(|&th| self.max_count(th) > 0).unwrap_or(MAX_LEVEL)
    }
    /// Army camp space at a level.
    pub fn housing(self, level: u8) -> u32 {
        match (self, level) {
            (Kind::ArmyCamp, 1..) => [24, 36, 48][usize::from(level.min(MAX_LEVEL) - 1)],
            _ => 0,
        }
    }
    pub fn is_defense(self) -> bool {
        matches!(self, Kind::Cannon | Kind::ArcherTower)
    }
}

/// The longest training queue.
pub const QUEUE_MAX: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Data)]
pub enum Troop {
    #[default]
    Fighter,
    Archer,
    Brute,
}

pub const TROOPS: [Troop; 3] = [Troop::Fighter, Troop::Archer, Troop::Brute];

impl Troop {
    pub fn key(self) -> &'static str {
        match self {
            Troop::Fighter => "fighter",
            Troop::Archer => "archer",
            Troop::Brute => "brute",
        }
    }
    pub fn from_key(key: &str) -> Option<Troop> {
        TROOPS.into_iter().find(|t| t.key() == key)
    }
    pub fn name(self) -> &'static str {
        match self {
            Troop::Fighter => "Fighter",
            Troop::Archer => "Archer",
            Troop::Brute => "Brute",
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
    /// Army camp space one takes.
    pub fn housing(self) -> u32 {
        match self {
            Troop::Brute => 6,
            _ => 1,
        }
    }
    /// Elixir to train one.
    pub fn cost(self) -> u32 {
        match self {
            Troop::Fighter => 30,
            Troop::Archer => 60,
            Troop::Brute => 280,
        }
    }
    pub fn train_secs(self) -> u64 {
        match self {
            Troop::Fighter => 6,
            Troop::Archer => 10,
            Troop::Brute => 24,
        }
    }
    /// The barracks level that unlocks it.
    pub fn barracks_level(self) -> u8 {
        match self {
            Troop::Brute => 2,
            _ => 1,
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

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Town {
    pub gold: u32,
    pub elixir: u32,
    pub gems: u32,
    pub knowledge: u32,
    pub buildings: Vec<Building>,
    pub next_id: u32,
    /// Trained troops, counted per `TROOPS` index.
    pub army: Vec<u32>,
    /// Troops waiting to train, the first one training.
    pub queue: Vec<Troop>,
    /// When the first troop in the queue started training; 0 when the queue is empty.
    pub queue_start: u64,
    /// Best stars (0 to 3) won at each campaign camp.
    pub stars: Vec<u8>,
    /// Per campaign camp, a bitmask of its buildings already looted.
    pub looted: Vec<u32>,
}

/// How many campaign camps there are (battle::CAMPS).
pub const CAMP_COUNT: usize = 4;

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
            army: vec![0; TROOPS.len()],
            queue: Vec::new(),
            queue_start: 0,
            stars: vec![0; CAMP_COUNT],
            looted: vec![0; CAMP_COUNT],
        }
    }

    /// The end of a raid on campaign camp `camp`: keep the best stars, mark
    /// what was looted, and credit the loot up to the storage caps (what
    /// doesn't fit is lost). Returns the (gold, elixir) actually kept.
    pub fn record_raid(&mut self, camp: usize, stars: u8, looted_now: u32, gold: u32, elixir: u32) -> (u32, u32) {
        if let Some(best) = self.stars.get_mut(camp) {
            *best = (*best).max(stars.min(3));
        }
        if let Some(mask) = self.looted.get_mut(camp) {
            *mask |= looted_now;
        }
        (self.earn(Res::Gold, gold), self.earn(Res::Elixir, elixir))
    }

    /// The highest finished barracks level (0 without one).
    pub fn barracks_level(&self) -> u8 {
        self.buildings
            .iter()
            .filter(|b| b.kind == Kind::Barracks)
            .map(|b| b.level)
            .max()
            .unwrap_or(0)
    }
    /// Army camp space from finished camps.
    pub fn housing_cap(&self) -> u32 {
        self.buildings.iter().map(|b| b.kind.housing(b.level)).sum()
    }
    /// Space taken by trained AND queued troops.
    pub fn housing_used(&self) -> u32 {
        let trained: u32 = TROOPS.iter().map(|t| self.trained(*t) * t.housing()).sum();
        trained + self.queue.iter().map(|t| t.housing()).sum::<u32>()
    }
    pub fn trained(&self, t: Troop) -> u32 {
        self.army.get(t.index()).copied().unwrap_or(0)
    }
    /// Why `t` can't be queued now.
    pub fn can_train(&self, t: Troop) -> Result<(), String> {
        let have = self.barracks_level();
        if have == 0 {
            return Err("Build a Barracks".into());
        }
        if have < t.barracks_level() {
            return Err(format!("Needs Barracks level {}", t.barracks_level()));
        }
        if self.queue.len() >= QUEUE_MAX {
            return Err("Queue full".into());
        }
        if self.housing_used() + t.housing() > self.housing_cap() {
            return Err("Army camps full".into());
        }
        self.afford(Res::Elixir, t.cost())
    }
    pub fn train(&mut self, t: Troop, now: u64) -> Result<(), String> {
        self.can_train(t)?;
        self.spend(Res::Elixir, t.cost());
        if self.queue.is_empty() {
            self.queue_start = now;
        }
        self.queue.push(t);
        Ok(())
    }
    /// Takes queued troop `i` out and refunds it in full. Cancelling the one
    /// in training restarts the next from now.
    pub fn cancel_training(&mut self, i: usize, now: u64) -> Result<(), String> {
        if i >= self.queue.len() {
            return Err("Nothing to cancel".into());
        }
        let t = self.queue.remove(i);
        self.earn_refund(t.cost());
        if i == 0 {
            self.queue_start = if self.queue.is_empty() { 0 } else { now };
        }
        Ok(())
    }
    /// A refund always comes back in full, even over the cap.
    fn earn_refund(&mut self, n: u32) {
        self.elixir = self.elixir.saturating_add(n);
    }
    /// Seconds until the troop in training is ready (rounded up), if any.
    pub fn training_left(&self, now: u64) -> u64 {
        match self.queue.first() {
            Some(t) => (self.queue_start + t.train_secs() * 1000).saturating_sub(now).div_ceil(1000),
            None => 0,
        }
    }
    /// Moves every troop whose training ended by `now` into the army.
    pub fn settle_training(&mut self, now: u64) {
        while let Some(&t) = self.queue.first() {
            let done = self.queue_start + t.train_secs() * 1000;
            if done > now {
                break;
            }
            self.queue.remove(0);
            self.army[t.index()] += 1;
            self.queue_start = if self.queue.is_empty() { 0 } else { done };
        }
    }
    /// Takes `n` of a trained troop out of the army (they go to battle).
    pub fn take_troops(&mut self, t: Troop, n: u32) -> u32 {
        let have = self.trained(t);
        let took = n.min(have);
        self.army[t.index()] = have - took;
        took
    }
    /// Dev only: fill both storages.
    pub fn dev_rich(&mut self) {
        let (gc, ec) = self.caps();
        self.gold = gc;
        self.elixir = ec;
    }
    /// Dev only: raise the Town Hall one level, instantly.
    pub fn dev_town_hall(&mut self, now: u64) {
        if let Some(i) = self.buildings.iter().position(|b| b.kind == Kind::TownHall) {
            let b = &mut self.buildings[i];
            if !b.busy() && b.level < MAX_LEVEL {
                b.level += 1;
                b.mark = now;
            }
        }
    }

    /// Dev only: an army camp if there is none, then fill it with Fighters.
    pub fn dev_army(&mut self, now: u64) {
        if self.count(Kind::ArmyCamp) == 0 {
            if let Some((x, z)) = self.free_spot(Kind::ArmyCamp) {
                let id = self.next_id;
                self.next_id += 1;
                self.buildings.push(Building { id, kind: Kind::ArmyCamp, level: 1, x, z, mark: now, ..Building::default() });
            }
        }
        let room = self.housing_cap().saturating_sub(self.housing_used());
        self.army[Troop::Fighter.index()] += room;
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
        self.settle_training(now);
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
        // Since v2: "#army counts,queue_start#queued troop indexes".
        let army: Vec<String> = self.army.iter().map(|n| n.to_string()).collect();
        let queue: Vec<String> = self.queue.iter().map(|t| t.index().to_string()).collect();
        out += &format!("#{},{}#{}", army.join(","), self.queue_start, queue.join(","));
        // Since the campaign: "#best stars per camp#looted masks per camp".
        let stars: Vec<String> = self.stars.iter().map(|n| n.to_string()).collect();
        let looted: Vec<String> = self.looted.iter().map(|n| n.to_string()).collect();
        out += &format!("#{}#{}", stars.join(","), looted.join(","));
        out
    }

    /// Reads `encode`'s text back; None if it isn't a village.
    pub fn decode(text: &str) -> Option<Town> {
        // A v1 village has no army part.
        let mut sections = text.split('#');
        let text = sections.next()?;
        let (a, q, st, lo) = (sections.next(), sections.next(), sections.next(), sections.next());
        if sections.next().is_some() {
            return None;
        }
        // An army-era village has no campaign part.
        let (stars, looted) = match (st, lo) {
            (None, None) => (vec![0; CAMP_COUNT], vec![0; CAMP_COUNT]),
            (Some(st), Some(lo)) => (
                st.split(',').map(|n| n.parse().ok()).collect::<Option<Vec<u8>>>()?,
                lo.split(',').map(|n| n.parse().ok()).collect::<Option<Vec<u32>>>()?,
            ),
            _ => return None,
        };
        let (army, queue_start, queue) = match (a, q) {
            (None, None) => (vec![0; TROOPS.len()], 0, Vec::new()),
            (Some(a), Some(q)) => {
                let nums: Vec<u64> = a.split(',').map(|n| n.parse().ok()).collect::<Option<_>>()?;
                let (start, counts) = nums.split_last()?;
                if counts.len() != TROOPS.len() {
                    return None;
                }
                let army = counts.iter().map(|&n| u32::try_from(n).ok()).collect::<Option<Vec<u32>>>()?;
                let queue = if q.is_empty() {
                    Vec::new()
                } else {
                    q.split(',')
                        .map(|n| n.parse::<usize>().ok().and_then(|i| TROOPS.get(i).copied()))
                        .collect::<Option<Vec<Troop>>>()?
                };
                (army, *start, queue)
            }
            _ => return None,
        };
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
        let town = Town {
            gold,
            elixir,
            gems,
            knowledge,
            buildings,
            next_id,
            army,
            queue,
            queue_start,
            stars,
            looted,
        };
        town.valid().then_some(town)
    }

    /// Whether a village could have come from play: unique ids below next_id,
    /// one Town Hall, counts and levels within limits, on the grid without
    /// overlaps, no more busy builders than builders, nothing over its cap.
    pub fn valid(&self) -> bool {
        let bs = &self.buildings;
        let ids_ok = bs.iter().enumerate().all(|(i, b)| {
            b.id != 0 && b.id < self.next_id && bs[..i].iter().all(|o| o.id != b.id)
        });
        let th = self.town_hall_level();
        let counts_ok = self.count(Kind::TownHall) == 1
            && KINDS.iter().all(|&k| self.count(k) <= k.max_count(MAX_LEVEL));
        let levels_ok = bs.iter().all(|b| {
            let top = if b.busy() { b.level + 1 } else { b.level.max(1) };
            top <= b.kind.max_level() && (b.kind == Kind::TownHall || b.level <= th)
        });
        let grid_ok = bs.iter().enumerate().all(|(i, b)| {
            let s = b.kind.size();
            (0..=GRID - s).contains(&b.x)
                && (0..=GRID - s).contains(&b.z)
                && bs[..i].iter().all(|o| {
                    let os = o.kind.size();
                    !(b.x < o.x + os && o.x < b.x + s && b.z < o.z + os && o.z < b.z + s)
                })
        });
        let banked_ok = bs.iter().all(|b| {
            let holds = b.kind.production(b.level).map_or(0, |(_, _, h)| h);
            b.banked_milli <= holds * 1000
        });
        let (gc, ec) = self.caps();
        ids_ok
            && counts_ok
            && levels_ok
            && grid_ok
            && banked_ok
            && self.builders_busy() <= self.builders()
            && self.army.len() == TROOPS.len()
            && self.stars.len() == CAMP_COUNT
            && self.stars.iter().all(|&s| s <= 3)
            && self.looted.len() == CAMP_COUNT
            && self.queue.len() <= QUEUE_MAX
            && self.housing_used() <= self.housing_cap()
            && self.next_id < u32::MAX
            && self.gold <= gc.max(START_GOLD)
            && self.elixir <= ec.max(START_ELIXIR)
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
