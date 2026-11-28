//! Battle rules (SPEC-2 G5): whole-number, deterministic, always ends.
use village_logic::battle::*;
use village_logic::economy::*;

fn run(b: &mut Battle) -> End {
    for _ in 0..=BATTLE_TICKS {
        if let Some(end) = b.step() {
            return end;
        }
    }
    panic!("the battle never ended");
}

fn plan(kind: Kind, x: i32, z: i32) -> Plan {
    Plan { kind, level: 1, x, z, gold: 0, elixir: 0 }
}

#[test]
fn fighters_clear_the_free_clinic_and_take_all_its_loot() {
    let mut b = Battle::new(0, vec![6, 0, 0], 0);
    for i in 0..6 {
        b.deploy(Troop::Fighter, 2 + i, 2).unwrap();
    }
    assert_eq!(run(&mut b), End::Destroyed);
    assert_eq!((b.percent(), b.stars()), (100, 3));
    let (gold, elixir) = CAMPS[0].1.iter().fold((0, 0), |(g, e), p| (g + p.gold, e + p.elixir));
    assert_eq!((b.gold, b.elixir), (gold, elixir));
    assert_eq!(b.looted_now(), 0b11111);
}

#[test]
fn a_replay_gives_no_loot_from_buildings_already_looted() {
    let mut b = Battle::new(0, vec![6, 0, 0], 0b00011);
    for i in 0..6 {
        b.deploy(Troop::Fighter, 2 + i, 2).unwrap();
    }
    run(&mut b);
    let rest = CAMPS[0].1[2..].iter().fold(0, |g, p| g + p.gold + p.elixir);
    assert_eq!(b.gold + b.elixir, rest);
    assert_eq!(b.looted_now(), 0b11100);
}

#[test]
fn a_cannon_shoots_troops_in_range_and_ignores_them_out_of_range() {
    let mut b = Battle::on(&[plan(Kind::Cannon, 20, 20)], vec![1, 0, 0], 0);
    b.deploy(Troop::Fighter, 0, 0).unwrap();
    let (full, ..) = stats(Troop::Fighter);
    // Far away (over 20 tiles), it is not shot while it walks in.
    for _ in 0..60 {
        b.step();
    }
    assert_eq!(b.units[0].hp, full);
    let end = run(&mut b);
    assert_eq!(end, End::Wiped, "one fighter loses to a cannon");
    assert!(b.targets[0].hp < b.targets[0].max_hp, "but it got some hits in");
}

#[test]
fn a_brute_goes_for_defenses_first() {
    let layout = [plan(Kind::GoldStorage, 3, 3), plan(Kind::Cannon, 20, 20)];
    let mut b = Battle::on(&layout, vec![0, 0, 1], 0);
    b.deploy(Troop::Brute, 1, 1).unwrap();
    for _ in 0..30 {
        b.step();
    }
    assert_eq!(b.units[0].target, Some(1), "walks past the storage to the cannon");
    let mut b2 = b.clone();
    while b2.targets[1].alive() && !b2.over {
        b2.step();
        assert_eq!(b2.targets[0].hp, b2.targets[0].max_hp, "storage untouched while the cannon stands");
    }
}

#[test]
fn stars_and_percent_follow_what_is_destroyed() {
    let mut b = Battle::new(0, vec![0, 0, 0], 0);
    assert_eq!((b.percent(), b.stars()), (0, 0));
    b.targets[1].hp = 0;
    b.targets[2].hp = 0;
    b.targets[3].hp = 0;
    assert_eq!((b.percent(), b.stars()), (60, 1));
    b.targets[0].hp = 0; // the Town Hall
    assert_eq!(b.stars(), 2);
    b.targets[4].hp = 0;
    assert_eq!((b.percent(), b.stars()), (100, 3));
}

#[test]
fn the_same_drops_give_the_same_fight() {
    let fight = || {
        let mut b = Battle::new(2, vec![10, 8, 2], 0);
        for i in 0..10 {
            b.deploy(Troop::Fighter, i, 0).unwrap();
        }
        for i in 0..8 {
            b.deploy(Troop::Archer, 29, i * 3).unwrap();
        }
        b.deploy(Troop::Brute, 0, 29).unwrap();
        b.deploy(Troop::Brute, 1, 29).unwrap();
        let end = run(&mut b);
        (end, b)
    };
    assert_eq!(fight(), fight());
}

#[test]
fn every_battle_ends_in_time_even_with_nothing_dropped() {
    let mut b = Battle::new(3, vec![5, 5, 1], 0);
    assert_eq!(run(&mut b), End::Time);
    assert_eq!(b.tick, BATTLE_TICKS);
}

#[test]
fn troops_dropped_on_one_tile_spread_out() {
    let mut b = Battle::new(0, vec![10, 0, 0], 0);
    for _ in 0..10 {
        b.deploy(Troop::Fighter, 1, 1).unwrap();
    }
    for _ in 0..30 {
        b.step();
    }
    let mut spots: Vec<(i64, i64)> = b.units.iter().map(|u| (u.x, u.z)).collect();
    spots.sort();
    spots.dedup();
    assert_eq!(spots.len(), 10, "no two troops share a spot");
}

#[test]
fn drops_inside_the_red_border_or_off_the_map_are_refused() {
    let b = Battle::new(0, vec![1, 0, 0], 0);
    // The Town Hall sits at (13, 13); one tile around it is the border.
    assert_eq!(b.can_deploy(Troop::Fighter, 12, 12), Err("Too close: drop outside the red border".into()));
    assert_eq!(b.can_deploy(Troop::Fighter, -1, 0), Err("Off the map".into()));
    assert_eq!(b.can_deploy(Troop::Archer, 0, 0), Err("No Archers left".into()));
    assert!(b.can_deploy(Troop::Fighter, 0, 0).is_ok());
}

#[test]
fn a_player_village_becomes_a_raidable_layout() {
    let town = Town::new();
    let layout = plans_from_town(&town);
    assert!(layout.iter().any(|p| p.kind == Kind::TownHall));
    let b = Battle::on(&layout, vec![1, 0, 0], 0);
    assert_eq!(b.targets.len(), layout.len());
}
