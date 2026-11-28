//! Step 1 balance (ROADMAP.md "Step 1"): mines hold hours, building HP grows
//! with level, camps refill once per Anki day, damage timing is locked, and a
//! seven-day sim with a fixed player strategy hits the pass bars.
use village_logic::battle::*;
use village_logic::economy::*;

const H: u64 = 3_600_000;
const DAY_MS: u64 = 24 * H;
/// How long the player stays on each touch.
const TOUCH_MS: u64 = 10 * 60_000;

// ---- Economy numbers ----

#[test]
fn every_mine_and_collector_holds_2_to_12_hours() {
    for kind in [Kind::GoldMine, Kind::ElixirCollector] {
        for level in 1..=kind.max_level() {
            let (_, per_hour, holds) = kind.production(level).unwrap();
            let hours = holds as f64 / per_hour as f64;
            assert!((2.0..=12.0).contains(&hours), "{kind:?} L{level} holds {hours:.1} h");
        }
    }
    // Production is per hour: a level-1 Mint makes its hourly rate in an hour, not a minute.
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.settle(10_000);
    let (_, per_hour, _) = Kind::GoldMine.production(1).unwrap();
    assert_eq!(u64::from(t.get(id).unwrap().waiting(10_000 + H)), per_hour);
    assert!(u64::from(t.get(id).unwrap().waiting(10_000 + 60_000)) < per_hour / 50);
}

#[test]
fn building_hp_at_level_8_is_at_least_3x_level_1() {
    for kind in KINDS {
        let (one, eight) = (building_hp(kind, 1), building_hp(kind, 8));
        assert!(eight >= 3 * one, "{kind:?}: {one} → {eight}");
        for level in 2..=8 {
            assert!(building_hp(kind, level) > building_hp(kind, level - 1));
        }
    }
    // Troops level with the Barracks.
    let (hp1, dmg1, ..) = troop_stats(Troop::Fighter, 1);
    let (hp5, dmg5, ..) = troop_stats(Troop::Fighter, 5);
    assert!(hp5 > hp1 && dmg5 > dmg1);
}

#[test]
fn camps_scale_with_the_keep_and_refill_once_per_anki_day() {
    // At Keep 1 a camp is its plan; at Keep 5 every building is 4 levels up with more loot.
    let one = camp_plan(3, 1);
    let five = camp_plan(3, 5);
    for (a, (b, plan)) in one.iter().zip(five.iter().zip(CAMPS[3].1)) {
        assert_eq!((a.level, a.gold, a.elixir), (plan.level, plan.gold, plan.elixir));
        assert_eq!(b.level, (plan.level + 4).min(8));
        assert!(b.gold >= a.gold && b.elixir >= a.elixir);
    }
    // 4 am local rollover, here in UTC-7 (4 am local = 11:00 UTC).
    let midnight = 1_764_288_000_000u64; // 2025-11-28 00:00 UTC
    let tz = -420;
    let before = review_day(midnight + 11 * H - 60_000, tz); // 03:59 local
    let after = review_day(midnight + 11 * H, tz); // 04:00 local
    assert_eq!(before + 1, after);
    let mut t = Town::new();
    assert!(t.refill_camps(before));
    t.record_raid(0, 3, 0b11111, 0, 0);
    // Later the same Anki day (still before 4 am): no refill.
    assert!(!t.refill_camps(before));
    assert_eq!(t.looted[0], 0b11111);
    // 4 am: refilled once, best stars kept; again that day does nothing.
    assert!(t.refill_camps(after));
    assert_eq!((t.looted[0], t.stars[0]), (0, 3));
    t.record_raid(0, 3, 0b00001, 0, 0);
    assert!(!t.refill_camps(after));
    assert_eq!(t.looted[0], 0b00001);
    // The refill day survives a save.
    assert_eq!(Town::decode(&t.encode()).unwrap().camp_day, after);
}

#[test]
fn a_fixed_seed_battle_gives_the_same_result_values() {
    // Locks damage timing (battle.rs): if these move, a rules change moved them.
    let mut b = Battle::camp(2, 2, 2, vec![20, 12, 2], 0);
    drop_in_groups(&mut b, [20, 12, 2], 7);
    let end = run(&mut b);
    let got = (end, b.tick, b.stars(), b.percent(), b.gold, b.elixir, b.units.iter().filter(|u| u.alive()).count());
    let again = {
        let mut b = Battle::camp(2, 2, 2, vec![20, 12, 2], 0);
        drop_in_groups(&mut b, [20, 12, 2], 7);
        let end = run(&mut b);
        (end, b.tick, b.stars(), b.percent(), b.gold, b.elixir, b.units.iter().filter(|u| u.alive()).count())
    };
    assert_eq!(got, again);
    println!("fixed battle: {got:?}");
    assert_eq!(got, FIXED_BATTLE);
}

/// The frozen result of `a_fixed_seed_battle_gives_the_same_result_values`.
const FIXED_BATTLE: (End, u32, u8, u32, u32, u32, usize) = (End::Wiped, 4131, 2, 75, 1400, 1900, 0);

// ---- The fixed strategy ----

fn run(b: &mut Battle) -> End {
    for _ in 0..=BATTLE_TICKS {
        if let Some(end) = b.step() {
            return end;
        }
    }
    panic!("the battle never ended");
}

/// A tiny deterministic generator for drop jitter.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo + 1) as u64) as i32
    }
}

/// The attacker's drop: the whole army in 3 equal groups on the map side
/// nearest the Keep, the groups spread along that side (seeded jitter).
fn drop_in_groups(b: &mut Battle, army: [u32; 3], seed: u64) {
    let mut rng = Lcg(seed.wrapping_mul(2654435761).wrapping_add(17));
    let keep = b.targets.iter().find(|t| t.kind == Kind::TownHall).map_or((15_000, 15_000), |t| t.centre());
    let (kx, kz) = ((keep.0 / MILLI) as i32, (keep.1 / MILLI) as i32);
    let last = GRID - 1;
    // Sides: 0 = x 0, 1 = x last, 2 = z 0, 3 = z last; nearest first, ties in that order.
    let dist = [kx, last - kx, kz, last - kz];
    let side = (0..4).min_by_key(|&s| (dist[s], s)).unwrap();
    let along_mid = if side < 2 { kz } else { kx };
    let spots: Vec<(i32, i32)> = [-7, 0, 7]
        .iter()
        .map(|off| {
            let along = (along_mid + off + rng.range(-2, 2)).clamp(0, last);
            let depth = rng.range(0, 1);
            match side {
                0 => (depth, along),
                1 => (last - depth, along),
                2 => (along, depth),
                _ => (along, last - depth),
            }
        })
        .collect();
    for (g, &(x, z)) in spots.iter().enumerate() {
        // Brutes first (they tank), then Spearmen, then Bowmen.
        for t in [Troop::Brute, Troop::Fighter, Troop::Archer] {
            let n = army[t.index()];
            let share = n / 3 + u32::from((g as u32) < n % 3);
            for _ in 0..share {
                place_near(b, t, x, z);
            }
        }
    }
}

/// Drops `t` at (x, z) or the nearest tile that takes it.
fn place_near(b: &mut Battle, t: Troop, x: i32, z: i32) {
    for r in 0..GRID {
        for dz in -r..=r {
            for dx in -r..=r {
                if dx.abs().max(dz.abs()) == r && b.deploy(t, x + dx, z + dz).is_ok() {
                    return;
                }
            }
        }
    }
}

/// A full army for `housing`: a fifth of the space in Lancers once the
/// Barracks trains them, the rest half Spearmen, half Bowmen.
fn full_army(housing: u32, barracks: u8) -> [u32; 3] {
    let brutes = if barracks >= Troop::Brute.barracks_level() { housing / 5 / Troop::Brute.housing() } else { 0 };
    let rest = housing - brutes * Troop::Brute.housing();
    let archers = rest / 2;
    [rest - archers, archers, brutes]
}

#[derive(Clone, Copy, Debug)]
enum Act {
    Up(u32),
    Buy(Kind),
}

/// The cheapest thing the builder can start now: an upgrade or a new building.
fn cheapest(t: &Town) -> Option<(Act, u32)> {
    let ups = t
        .buildings
        .iter()
        .filter(|b| t.can_upgrade(b.id).is_ok())
        .map(|b| (Act::Up(b.id), b.kind.cost(b.level + 1).1, b.kind as u32, b.id));
    let buys = KINDS
        .into_iter()
        .filter(|&k| t.can_buy(k).is_ok() && t.free_spot(k).is_some())
        .map(|k| (Act::Buy(k), k.cost(1).1, k as u32, 0));
    ups.chain(buys).min_by_key(|&(_, cost, kind, id)| (cost, kind, id)).map(|(a, c, ..)| (a, c))
}

/// Grants what 30 credited minutes pay under ANTICHEAT v3: 30 Knowledge, the
/// 10- and 30-minute chests, the queue-cleared chest (the size of the
/// 30-minute one) and 1 power hour, through the game's own chest and power
/// code. Test-side stand-in: step 4 replaces review counts with credited
/// minutes in the game itself.
fn credit_30_minutes(t: &mut Town, now: u64) {
    t.knowledge += 30;
    t.pay_chest(0, 0);
    t.pay_chest(1, 0);
    t.pay_chest(1, 0);
    t.start_power(now, 1);
}

struct Log {
    keep_by_day: Vec<u8>,
    raids: u32,
    raid_stars: u32,
    dead_end: Option<String>,
}

/// One week of the fixed strategy. The player starts day 1 at 08:00 and
/// touches the game at 08:00, 12:00, 13:00, 16:00 and 20:00: every touch
/// collects everything (so at least every 4 h while awake) and starts the
/// cheapest thing affordable until nothing is (staying up to 10 minutes while
/// jobs finish and free a mason); the three sessions (08:00,
/// 13:00, 20:00) also raid a campaign camp with a full army and train the next.
/// With study, 30 credited minutes land at 20:00 and Knowledge finishes timers.
fn week(study: bool) -> (Town, Log) {
    let mut t = Town::new();
    let mut log = Log { keep_by_day: Vec::new(), raids: 0, raid_stars: 0, dead_end: None };
    let touches = [(8, true), (12, false), (13, true), (16, false), (20, true)];
    let mut progress: Vec<u64> = Vec::new();
    for day in 0..7u32 {
        for &(hour, session) in &touches {
            let now = u64::from(day) * DAY_MS + (hour - 8) * H;
            t.settle(now);
            if hour == 8 {
                t.refill_camps(day + 1);
            }
            t.collect_all(now);
            if study && hour == 20 {
                credit_30_minutes(&mut t, now);
            }
            if session {
                raid(&mut t, now, u64::from(day) * 10 + hour, &mut log);
                train(&mut t, now);
            }
            // Progress moments: something affordable now, and every job's finish.
            if cheapest(&t).is_some() {
                progress.push(now);
            }
            // A touch lasts up to 10 minutes: jobs that finish meanwhile free a
            // mason for the next cheapest thing.
            let mut at = now;
            loop {
                build(&mut t, at, study);
                let next = t.buildings.iter().filter(|b| b.busy()).map(|b| b.done_at).min();
                match next {
                    Some(done) if done <= now + TOUCH_MS => {
                        at = done;
                        t.settle(at);
                        t.collect_all(at);
                    }
                    _ => break,
                }
            }
            progress.extend(t.buildings.iter().filter(|b| b.busy()).map(|b| b.done_at));
        }
        t.settle(u64::from(day + 1) * DAY_MS);
        log.keep_by_day.push(t.town_hall_level());
        // SIMDBG=1 prints each day's village (for retuning).
        if std::env::var("SIMDBG").is_ok() {
            let lv: Vec<String> = t.buildings.iter().map(|b| format!("{}{}{}", b.kind.key(), b.level, if b.busy() { format!("*{:.1}h", (b.done_at as f64 - (u64::from(day + 1) * DAY_MS) as f64) / H as f64) } else { String::new() })).collect();
            println!("d{} study={study} g={} e={} caps={:?} k={} army={:?} | {}", day + 1, t.gold, t.elixir, t.caps(), t.knowledge, t.army, lv.join(" "));
        }
    }
    // No dead end: no 24 h window of the week without a progress moment.
    progress.push(0);
    progress.sort_unstable();
    progress.dedup();
    let end = 7 * DAY_MS;
    let mut last = 0;
    for &p in progress.iter().filter(|&&p| p <= end).chain([end].iter()) {
        if p - last > DAY_MS && log.dead_end.is_none() {
            log.dead_end = Some(format!("nothing from {:.1} h to {:.1} h", last as f64 / H as f64, p as f64 / H as f64));
        }
        last = p;
    }
    (t, log)
}

fn build(t: &mut Town, now: u64, study: bool) {
    loop {
        while let Some((act, _)) = cheapest(t) {
            match act {
                Act::Up(id) => t.upgrade(id, now).unwrap(),
                Act::Buy(k) => {
                    let (x, z) = t.free_spot(k).unwrap();
                    t.place(k, x, z, now).unwrap();
                }
            }
        }
        if !study {
            return;
        }
        // Knowledge finishes the biggest timer it can pay for whole.
        let best = t
            .buildings
            .iter()
            .filter_map(|b| t.skip_cost(b.id, now, Pay::Knowledge).filter(|&c| c <= t.knowledge).map(|c| (c, b.id)))
            .max();
        let Some((_, id)) = best else { return };
        t.skip(id, now, Pay::Knowledge).unwrap();
    }
}

fn train(t: &mut Town, now: u64) {
    let want = full_army(t.housing_cap(), t.barracks_level());
    for tr in [Troop::Brute, Troop::Fighter, Troop::Archer] {
        let have = t.trained(tr) + t.queue.iter().filter(|q| **q == tr).count() as u32;
        for _ in have..want[tr.index()] {
            if t.train(tr, now).is_err() {
                break;
            }
        }
    }
}

/// Raids the camp with the most loot left, if the whole army is trained.
fn raid(t: &mut Town, now: u64, seed: u64, log: &mut Log) {
    t.settle_training(now);
    let cap = t.housing_cap();
    if cap == 0 || !t.queue.is_empty() || t.housing_used() < cap {
        return;
    }
    let keep = t.town_hall_level();
    let left = |c: usize| -> u32 {
        camp_plan(c, keep)
            .iter()
            .enumerate()
            .filter(|(i, _)| t.looted[c] & (1 << i) == 0)
            .map(|(_, p)| p.gold + p.elixir)
            .sum()
    };
    let Some(camp) = (0..CAMP_COUNT).filter(|&c| left(c) > 0).max_by_key(|&c| (left(c), c)) else { return };
    let army = [t.trained(Troop::Fighter), t.trained(Troop::Archer), t.trained(Troop::Brute)];
    let mut b = Battle::camp(camp, keep, t.barracks_level(), army.to_vec(), t.looted[camp]);
    drop_in_groups(&mut b, army, seed);
    for tr in TROOPS {
        t.take_troops(tr, army[tr.index()]);
    }
    run(&mut b);
    t.record_raid(camp, b.stars(), b.looted_now(), b.gold, b.elixir);
    log.raids += 1;
    log.raid_stars += u32::from(b.stars());
}

fn levels(t: &Town) -> u32 {
    t.buildings.iter().map(|b| u32::from(b.level)).sum()
}

/// 50 seeded fights: the base's own full army (its Barracks level) against
/// the base itself. Returns (average stars, 3-star rate).
fn level_matched(base: &Town) -> (f64, f64, f64) {
    let plan = plans_from_town(base);
    let army = full_army(base.housing_cap(), base.barracks_level());
    let (mut stars, mut threes, mut pct) = (0u32, 0u32, 0u32);
    for seed in 0..50 {
        let mut b = Battle::on(&plan, army.to_vec(), 0);
        b.troop_level = base.barracks_level();
        drop_in_groups(&mut b, army, seed);
        run(&mut b);
        stars += u32::from(b.stars());
        threes += u32::from(b.stars() == 3);
        pct += b.percent();
    }
    (f64::from(stars) / 50.0, f64::from(threes) / 50.0, f64::from(pct) / 50.0)
}

#[test]
fn seven_day_sim() {
    let (plain, plain_log) = week(false);
    let (studied, study_log) = week(true);
    let (lp, ls) = (levels(&plain), levels(&studied));
    let gain = f64::from(ls) / f64::from(lp) - 1.0;
    println!("keep by day, 0 min:  {:?}", plain_log.keep_by_day);
    println!("keep by day, 30 min: {:?}", study_log.keep_by_day);
    println!("upgrade levels day 7: 0 min {lp}, 30 min {ls} (+{:.0}%)", gain * 100.0);
    println!(
        "camp raids: 0 min {} (avg {:.2} stars), 30 min {} (avg {:.2} stars)",
        plain_log.raids,
        f64::from(plain_log.raid_stars) / f64::from(plain_log.raids.max(1)),
        study_log.raids,
        f64::from(study_log.raid_stars) / f64::from(study_log.raids.max(1)),
    );
    let mut raid_stats = Vec::new();
    for (name, base) in [("0 min", &plain), ("30 min", &studied)] {
        let (avg, three, pct) = level_matched(base);
        println!(
            "level-matched raids on the {name} day-7 base (keep {}, {} buildings, army {:?}, troop L{}): avg {avg:.2} stars, 3-star {:.0}%, avg {pct:.0}%",
            base.town_hall_level(),
            base.buildings.len(),
            full_army(base.housing_cap(), base.barracks_level()),
            base.barracks_level(),
            three * 100.0
        );
        raid_stats.push((name, avg, three));
    }
    for (name, log) in [("0 min", &plain_log), ("30 min", &study_log)] {
        let k = &log.keep_by_day;
        assert!((2..=3).contains(&k[0]), "{name}: day 1 Keep {}", k[0]);
        assert!((3..=5).contains(&k[2]), "{name}: day 3 Keep {}", k[2]);
        assert!((5..=7).contains(&k[6]), "{name}: day 7 Keep {}", k[6]);
        assert!(k.iter().all(|&l| l < MAX_LEVEL), "{name}: maxed by day 7");
        assert!(log.dead_end.is_none(), "{name}: dead end at {:?}", log.dead_end);
    }
    assert!(gain >= 0.15, "30 credited minutes a day give only +{:.1}% upgrade levels", gain * 100.0);
    for (name, avg, three) in raid_stats {
        assert!((1.0..=2.0).contains(&avg), "{name}: level-matched average {avg:.2} stars");
        assert!(three <= 0.20, "{name}: 3-star rate {:.0}%", three * 100.0);
    }
}
