//! ROADMAP step 3 (readable battles): HP bars and shots are drawing only. For
//! fixed drops the result (stars, % destroyed, loot, end) must stay exactly
//! what battle.rs gave before the step. The expected values were captured from
//! the code at commit 98d011d, before any step-3 edit.
use village_logic::battle::*;
use village_logic::economy::*;

/// Runs to the end; also folds every tick's hit points into one number, so a
/// change in when any hit lands (not only the final result) shows up.
fn run(b: &mut Battle) -> (End, u64) {
    let mut trace: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |v: i64| {
        trace ^= v as u64;
        trace = trace.wrapping_mul(0x100_0000_01b3);
    };
    for _ in 0..=BATTLE_TICKS {
        let end = b.step();
        for u in &b.units {
            mix(i64::from(u.hp));
        }
        for t in &b.targets {
            mix(i64::from(t.hp));
        }
        if let Some(end) = end {
            return (end, trace);
        }
    }
    panic!("the battle never ended");
}

/// (camp, keep, troop level, reserve, drops) → the battle after it ends.
fn fight(camp: usize, keep: u8, level: u8, reserve: [u32; 3], drops: &[(Troop, i32, i32)]) -> (Battle, (End, u64)) {
    let mut b = Battle::camp(camp, keep, level, reserve.to_vec(), 0);
    for &(t, x, z) in drops {
        b.deploy(t, x, z).unwrap();
    }
    let end = run(&mut b);
    (b, end)
}

/// `n` drops in a row going east from (x, z).
fn line(t: Troop, x: i32, z: i32, n: i32) -> Vec<(Troop, i32, i32)> {
    (0..n).map(|i| (t, x + i, z)).collect()
}

/// `n` drops in a column going south from (x, z).
fn column(t: Troop, x: i32, z: i32, n: i32) -> Vec<(Troop, i32, i32)> {
    (0..n).map(|i| (t, x, z + i)).collect()
}

fn summary(b: &Battle, (end, trace): (End, u64)) -> String {
    format!("{end:?} trace={trace:#x} stars={} pct={} gold={} elixir={} looted={:#b} tick={}", b.stars(), b.percent(), b.gold, b.elixir, b.looted_now(), b.tick)
}

fn cases() -> Vec<(&'static str, Battle, (End, u64))> {
    let mut out = Vec::new();
    // Bandit Camp (no defenses): ten Spearmen and five Archers clear it.
    let mut d = line(Troop::Fighter, 2, 2, 10);
    d.extend(column(Troop::Archer, 27, 5, 5));
    let (b, e) = fight(0, 1, 1, [10, 5, 0], &d);
    out.push(("bandit", b, e));
    // Raider Outpost (one cannon): six Spearmen and four Archers from the north.
    let mut d = line(Troop::Fighter, 10, 2, 6);
    d.extend(line(Troop::Archer, 11, 3, 4));
    let (b, e) = fight(1, 1, 1, [6, 4, 0], &d);
    out.push(("outpost", b, e));
    // Rebel Fort (two cannons, a tower): a mixed wave from two sides.
    let mut d = line(Troop::Brute, 2, 2, 2);
    d.extend(line(Troop::Fighter, 2, 25, 8));
    d.extend(column(Troop::Archer, 27, 2, 6));
    let (b, e) = fight(2, 1, 1, [8, 6, 2], &d);
    out.push(("fort", b, e));
    // Warlord's Citadel at Keep 3, troop level 2: a big wave.
    let mut d = line(Troop::Brute, 1, 1, 4);
    d.extend(line(Troop::Fighter, 1, 26, 12));
    d.extend(column(Troop::Archer, 28, 1, 10));
    let (b, e) = fight(3, 3, 2, [12, 10, 4], &d);
    out.push(("citadel", b, e));
    // Raider Outpost at troop level 4: a big wave wins.
    let mut d = line(Troop::Brute, 3, 1, 3);
    d.extend(line(Troop::Fighter, 3, 27, 15));
    d.extend(column(Troop::Archer, 1, 4, 12));
    let (b, e) = fight(1, 1, 4, [15, 12, 3], &d);
    out.push(("outpost-big", b, e));
    out
}

/// Captured from the code at 98d011d (before step 3), never edited to match.
const BEFORE: [(&str, &str); 5] = [
    ("bandit", "Destroyed trace=0x35108c74d083b7ad stars=3 pct=100 gold=450 elixir=450 looted=0b11111 tick=1393"),
    ("outpost", "Wiped trace=0xb817bf03564d2c44 stars=0 pct=0 gold=0 elixir=0 looted=0b0 tick=421"),
    ("fort", "Wiped trace=0x334c1875a9dab6cf stars=0 pct=12 gold=500 elixir=0 looted=0b10000 tick=928"),
    ("citadel", "Wiped trace=0x8efcf48fcc0c54e2 stars=0 pct=0 gold=0 elixir=0 looted=0b0 tick=824"),
    ("outpost-big", "Destroyed trace=0xf64a4fcccaffacf8 stars=3 pct=100 gold=650 elixir=650 looted=0b111111 tick=768"),
];

#[test]
fn fixed_drops_give_the_same_stars_percent_and_loot_as_before_step_3() {
    let got: Vec<(&str, String)> = cases().into_iter().map(|(n, b, e)| (n, summary(&b, e))).collect();
    assert_eq!(got.len(), BEFORE.len());
    for ((name, now), (was_name, was)) in got.iter().zip(BEFORE) {
        assert_eq!(*name, was_name);
        assert_eq!(now, was, "{name}: the battle result changed");
    }
}
