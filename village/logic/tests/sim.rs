use exact_game::*;
use village_logic::economy::*;
use village_logic::*;

const S: u64 = 1000;
const MIN: u64 = 60 * S;
const HOUR: u64 = 60 * MIN;

fn town_with(gold: u32, elixir: u32) -> Town {
    let mut t = Town::new();
    t.gold = gold;
    t.elixir = elixir;
    t
}

// ---- F1: the starting village ----

#[test]
fn starts_with_a_town_hall_in_the_middle_and_resources() {
    let t = Town::new();
    assert_eq!(t.buildings.len(), 1);
    let th = &t.buildings[0];
    assert_eq!((th.kind, th.level), (Kind::TownHall, 1));
    assert_eq!((th.x, th.z), (13, 13));
    assert_eq!((t.gold, t.elixir, t.gems, t.knowledge), (500, 500, 50, 0));
    assert_eq!(t.caps(), (1000, 1000));
    assert_eq!(t.builders(), 1);
}

#[test]
fn the_world_publishes_the_hud() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(100.0);
    let w = sim.world();
    assert_eq!(w.resource::<Town>().gold, 500);
    let h = hud(w, now_ms(w), String::new());
    assert_eq!((h.gold, h.gold_cap, h.builders, h.town_hall), (500, 1000, 1, 1));
    assert_eq!(h.shop.len(), KINDS.len() - 1);
}

// ---- F2: shop and placement ----

#[test]
fn placing_takes_the_cost_and_a_builder() {
    let mut t = Town::new();
    let id = t.place(Kind::ElixirCollector, 2, 2, 0).unwrap();
    assert_eq!(t.gold, 350);
    assert_eq!(t.builders_free(), 0);
    let b = t.get(id).unwrap();
    assert_eq!((b.level, b.busy()), (0, true));
}

#[test]
fn cannot_overlap_or_go_off_the_grid() {
    let mut t = town_with(900, 900);
    // The Town Hall covers 13..17.
    assert_eq!(t.can_place(Kind::GoldMine, 11, 11).unwrap_err(), "Something is in the way");
    assert_eq!(t.can_place(Kind::GoldMine, 28, 0).unwrap_err(), "Off the edge");
    assert_eq!(t.can_place(Kind::GoldMine, -1, 0).unwrap_err(), "Off the edge");
    assert!(t.can_place(Kind::GoldMine, 10, 10).is_ok());
    assert!(t.can_place(Kind::GoldMine, 27, 27).is_ok());
    t.place(Kind::BuilderHut, 0, 0, 0).unwrap();
    assert_eq!(t.can_place(Kind::GoldMine, 1, 1).unwrap_err(), "Something is in the way");
    assert!(t.can_place(Kind::GoldMine, 2, 0).is_ok());
}

#[test]
fn cannot_buy_without_resources() {
    let t = town_with(100, 100);
    assert_eq!(t.can_buy(Kind::GoldMine).unwrap_err(), "Need 150 grain");
    let mut t = town_with(100, 100);
    assert!(t.place(Kind::GoldMine, 0, 0, 0).is_err());
    assert_eq!((t.gold, t.elixir), (100, 100));
}

#[test]
fn town_hall_level_caps_how_many() {
    let mut t = town_with(1000, 1000);
    t.place(Kind::BuilderHut, 0, 0, 0).unwrap();
    t.place(Kind::GoldMine, 3, 0, 0).unwrap();
    t.settle(60 * S);
    assert_eq!(t.can_buy(Kind::GoldMine).unwrap_err(), "Upgrade the Keep");
    assert_eq!(t.can_buy(Kind::BuilderHut).unwrap_err(), "Max built");
    assert_eq!(t.can_buy(Kind::TownHall).unwrap_err(), "Max built");
}

// ---- F3: production and collecting ----

#[test]
fn mines_produce_at_their_rate_up_to_their_cap() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    // Built at 10 s; makes 250 gold an hour from then; holds 10 hours (2,500).
    t.settle(10 * S);
    assert_eq!(t.get(id).unwrap().waiting(10 * S), 0);
    assert_eq!(t.get(id).unwrap().waiting(10 * S + 36 * MIN), 150);
    assert_eq!(t.get(id).unwrap().waiting(10 * S + 36 * MIN + 499), 150);
    assert_eq!(t.get(id).unwrap().waiting(10 * S + 1000 * HOUR), 2500);
}

#[test]
fn collecting_moves_into_storage_and_overflow_stays() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.gold = 950; // cap 1000
    let at = 10 * S + 12 * HOUR; // 12 h of 250/h = 3,000, held at 2,500
    let got = t.collect(id, at);
    assert_eq!(got, 50);
    assert_eq!(t.gold, 1000);
    assert_eq!(t.get(id).unwrap().waiting(at), 2450);
    t.gold = 0;
    assert_eq!(t.collect(id, at), 1000);
    assert_eq!(t.get(id).unwrap().waiting(at), 1450);
}

#[test]
fn collecting_often_loses_nothing() {
    let mut a = Town::new();
    let mut b = Town::new();
    let ia = a.place(Kind::GoldMine, 0, 0, 0).unwrap();
    let ib = b.place(Kind::GoldMine, 0, 0, 0).unwrap();
    a.gold = 0;
    b.gold = 0;
    // a collects every 7.333 s for an hour; b once at the end.
    let mut t = 10 * S;
    while t < 10 * S + HOUR {
        a.collect(ia, t);
        t += 7_333;
    }
    a.collect(ia, 10 * S + HOUR);
    b.collect(ib, 10 * S + HOUR);
    assert_eq!(a.gold, 250);
    assert_eq!(b.gold, 250);
}

#[test]
fn storages_raise_the_cap_once_built() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldStorage, 0, 0, 0).unwrap();
    assert_eq!(t.caps().0, 1000);
    t.settle(15 * S);
    assert_eq!(t.get(id).unwrap().level, 1);
    assert_eq!(t.caps(), (2500, 1000));
}

// ---- F4: builders and timers ----

#[test]
fn one_builder_means_one_job_and_a_hut_adds_a_second() {
    let mut t = town_with(2000, 2000);
    t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    assert_eq!(t.can_buy(Kind::ElixirCollector).unwrap_err(), "No free mason");
    // The hut is instant, needs no builder, and adds one.
    t.place(Kind::BuilderHut, 5, 0, 0).unwrap();
    assert_eq!((t.builders(), t.builders_free()), (2, 1));
    t.place(Kind::ElixirCollector, 8, 0, 0).unwrap();
    assert_eq!(t.builders_free(), 0);
    let th = t.buildings[0].id;
    assert_eq!(t.can_upgrade(th).unwrap_err(), "No free mason");
    assert_eq!(t.builders_busy(), 2);
}

#[test]
fn timers_finish_at_the_right_time_and_levels_matter() {
    let mut t = town_with(3000, 3000);
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.settle(9_999);
    assert_eq!(t.get(id).unwrap().level, 0);
    t.settle(10 * S);
    assert_eq!(t.get(id).unwrap().level, 1);
    // A mine can't pass the Town Hall's level.
    assert_eq!(t.can_upgrade(id).unwrap_err(), "Upgrade the Keep first");
    let th = t.buildings[0].id;
    t.upgrade(th, 10 * S).unwrap();
    assert_eq!(t.gold, 2000);
    t.settle(70 * S);
    assert_eq!(t.town_hall_level(), 2);
    assert_eq!(t.caps(), (2000, 2000));
    t.upgrade(id, 70 * S).unwrap();
    t.settle(100 * S);
    let b = t.get(id).unwrap();
    assert_eq!(b.level, 2);
    // It banked 60 s of 250/h (4.17) when the upgrade started and kept that;
    // level 2 adds 320 an hour.
    assert_eq!(b.waiting(100 * S + HOUR), 324);
}

#[test]
fn nothing_is_made_while_upgrading_but_banked_stays() {
    let mut t = town_with(3000, 3000);
    let th = t.buildings[0].id;
    t.upgrade(th, 0).unwrap();
    t.settle(60 * S);
    let id = t.place(Kind::GoldMine, 0, 0, 60 * S).unwrap();
    t.settle(70 * S);
    // 250 gold made in its first hour, then upgrading for 30 s.
    let up = 70 * S + HOUR;
    t.upgrade(id, up).unwrap();
    assert_eq!(t.get(id).unwrap().waiting(up + 20 * S), 250);
    t.settle(up + 30 * S);
    assert_eq!(t.get(id).unwrap().waiting(up + 30 * S), 250);
    assert_eq!(t.get(id).unwrap().waiting(up + 30 * S + HOUR), 570);
}

#[test]
fn max_level_stops_upgrades() {
    let mut t = town_with(100_000, 100_000);
    t.elixir = 100_000;
    let th = t.buildings[0].id;
    let mut now = 0;
    t.gold = 1_000_000;
    for _ in 0..7 {
        t.upgrade(th, now).unwrap();
        now += 30_000 * S;
        t.settle(now);
        // Storage caps stop gold from going above the cap; refill for the test.
        t.gold = 1_000_000;
    }
    assert_eq!(t.town_hall_level(), 8);
    assert_eq!(t.can_upgrade(th).unwrap_err(), "Max level");
}

// ---- F5: time away ----

#[test]
fn time_away_equals_playing_through() {
    let build = |t: &mut Town| {
        t.place(Kind::GoldMine, 0, 0, 0).unwrap();
        t.place(Kind::BuilderHut, 5, 0, 0).unwrap();
        t.place(Kind::ElixirCollector, 8, 0, 0).unwrap();
    };
    let mut a = town_with(1000, 1000);
    let mut b = town_with(1000, 1000);
    build(&mut a);
    build(&mut b);
    // a lives through every second; b jumps 10 minutes.
    for s in 0..=600 {
        a.settle(s * S);
    }
    b.settle(600 * S);
    a.collect_all(600 * S);
    b.collect_all(600 * S);
    assert_eq!(a.buildings, b.buildings);
    assert_eq!((a.gold, a.elixir), (b.gold, b.elixir));
}

#[test]
fn the_world_counts_time_away() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(100.0);
    sim.post("buy goldmine");
    sim.run(100.0);
    sim.post("place");
    sim.run(100.0);
    let id = 2; // the first building after the Town Hall
    assert_eq!(sim.world().resource::<Town>().get(id).unwrap().level, 0);
    sim.post("away 3600");
    sim.run(100.0);
    let w = sim.world();
    let b = w.resource::<Town>().get(id).unwrap().clone();
    assert_eq!(b.level, 1);
    assert!(b.waiting(now_ms(w)) >= 240, "most of an hour at 250/h");
}

#[test]
fn a_save_restores_the_village() {
    let mut sim: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();
    sim.run(100.0);
    sim.post("buy goldmine");
    sim.run(100.0);
    sim.post("place");
    sim.run(500.0);
    let before = sim.world().resource::<Town>().buildings.clone();
    let save = sim.save().unwrap();
    let mut again: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();
    again.restore(&save).unwrap();
    assert_eq!(again.world().resource::<Town>().buildings, before);
}

// ---- F6: speed-ups ----

#[test]
fn skipping_costs_gems_or_knowledge_by_time_left() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap(); // 10 s
    assert_eq!(t.skip_cost(id, 0, Pay::Gems), Some(1));
    assert_eq!(t.skip_cost(id, 0, Pay::Knowledge), Some(1));
    assert_eq!(t.skip(id, 0, Pay::Knowledge).unwrap_err(), "Need 1 Knowledge");
    t.knowledge += 1;
    t.skip(id, 0, Pay::Knowledge).unwrap();
    assert_eq!((t.knowledge, t.get(id).unwrap().level), (0, 1));
    assert_eq!(t.builders_free(), 1);
    assert_eq!(t.skip_cost(id, 0, Pay::Gems), None);
}

#[test]
fn gem_skip_rounds_up() {
    let mut t = town_with(5000, 5000);
    let th = t.buildings[0].id;
    t.upgrade(th, 0).unwrap(); // 60 s
    assert_eq!(t.skip_cost(th, 0, Pay::Gems), Some(6));
    assert_eq!(t.skip_cost(th, 59_001, Pay::Gems), Some(1));
    assert_eq!(t.skip_cost(th, 0, Pay::Knowledge), Some(1), "1 Knowledge = 5 minutes");
    assert_eq!(KNOWLEDGE_SECONDS, 300);
    t.skip(th, 1000, Pay::Gems).unwrap();
    assert_eq!((t.gems, t.town_hall_level()), (44, 2));
}

#[test]
fn knowledge_only_comes_from_reviews_and_dev_needs_the_switch() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.post("away 3600");
    sim.run(100.0);
    assert_eq!(sim.world().resource::<Town>().knowledge, 0);
    // Testers can't use the test review button: off until the hidden dev switch.
    sim.post("review");
    sim.post("dev rich");
    sim.run(100.0);
    assert_eq!(sim.world().resource::<Town>().knowledge, 0);
    assert!(sim.world().resource::<Town>().gold < 5000);
    sim.post("devmode on");
    for _ in 0..10 {
        sim.post("review");
    }
    sim.run(100.0);
    assert_eq!(sim.world().resource::<Town>().knowledge, 2, "1 Knowledge per 5 reviews");
}

// ---- Picking ----

#[test]
fn picking_finds_the_tile_under_the_middle_of_the_screen() {
    let view = Vec2::new(1440.0, 756.0);
    // The camera looks at LOOK, the screen's centre: x=3, z=3 → tile (18, 18).
    assert_eq!(tile_at(view, view / 2.0, &CamView::home()), Some((18, 18)));
    // Corner-on view: left of centre is lower x; lower on screen is nearer the camera (higher x and z).
    let (lx, _) = tile_at(view, Vec2::new(500.0, 378.0), &CamView::home()).unwrap();
    assert!(lx < 18);
    let (nx, nz) = tile_at(view, Vec2::new(720.0, 600.0), &CamView::home()).unwrap();
    assert!(nx > 18 && nz > 18);
    // The sky is not a tile.
    assert_eq!(tile_at(view, Vec2::new(720.0, 2.0), &CamView::home()), None);
}

#[test]
fn tapping_while_placing_moves_the_ghost_and_tapping_a_building_selects_it() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(50.0);
    sim.post("buy goldmine");
    sim.run(50.0);
    sim.post("tap 3 3");
    sim.run(50.0);
    {
        let ui = sim.world().resource::<Ui>();
        assert_eq!((ui.ghost_x, ui.ghost_z), (2, 2));
    }
    sim.post("place");
    sim.run(50.0);
    sim.post("cancel");
    sim.post("tap 14 14");
    sim.run(50.0);
    let w = sim.world();
    let th = w.resource::<Town>().buildings[0].id;
    assert_eq!(w.resource::<Ui>().selected, th);
    assert_eq!(w.resource::<Town>().get(2).map(|b| (b.x, b.z)), Some((2, 2)));
}

#[test]
fn a_town_round_trips_through_its_save_text() {
    let mut t = town_with(1000, 1000);
    t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.place(Kind::BuilderHut, 5, 0, 0).unwrap();
    t.add_reviews(260, 20_000, 0);
    assert!(t.study.reviews == 260 && t.study.power_until > 0);
    let text = t.encode();
    assert_eq!(Town::decode(&text).unwrap().study, t.study);
    let back = Town::decode(&text).unwrap();
    assert_eq!(back.encode(), text);
    assert_eq!(back.buildings, t.buildings);
    assert_eq!((back.gold, back.elixir, back.gems, back.knowledge, back.next_id), (t.gold, t.elixir, t.gems, t.knowledge, t.next_id));
    assert!(Town::decode("garbage").is_none());
    assert!(Town::decode("").is_none());
}

#[test]
fn the_world_saves_after_loading_and_a_reload_counts_time_away() {
    // First visit: nothing saved yet.
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.post("load 1000000 ");
    sim.run(100.0);
    sim.post("buy goldmine");
    sim.run(100.0);
    sim.post("place");
    sim.run(500.0);
    let saved = sim.world().resource::<Shown>().last_save.clone();
    assert!(saved.starts_with("save v2|"), "{saved}");
    let id = 2; // the mine
    // The page keeps it stamped with the real time; come back 11 hours later.
    let kept = format!("{}@5000000", &saved["save ".len()..]);
    let mut again = Sim::<Village>::new(Options::default()).unwrap();
    again.post(format!("load {} {kept}", 5_000_000 + 11 * HOUR));
    again.run(100.0);
    let w = again.world();
    let b = w.resource::<Town>().get(id).unwrap().clone();
    assert_eq!(b.level, 1, "the 10 s build finished while away");
    assert_eq!(b.waiting(now_ms(w)), 2500, "and the mine filled up (10 hours' worth)");
    assert_eq!(w.resource::<Town>().elixir, 350);
}

#[test]
fn nothing_is_saved_before_the_load_arrives() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(3000.0);
    assert_eq!(sim.world().resource::<Shown>().last_save, "");
}

#[test]
fn a_reload_after_no_time_adds_nothing_and_bad_saves_start_fresh() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.post("load 1000 ");
    sim.post("buy goldmine");
    sim.run(100.0);
    sim.post("place");
    sim.run(2000.0);
    let saved = sim.world().resource::<Shown>().last_save.clone();
    let then: u64 = saved.split('|').nth(1).unwrap().parse().unwrap();
    let mut again = Sim::<Village>::new(Options::default()).unwrap();
    again.post(format!("load 9000 {}@9000", &saved["save ".len()..]));
    again.run(34.0);
    let w = again.world();
    assert!(now_ms(w) >= then && now_ms(w) <= then + 100, "time continues, no jump");
    assert_eq!(w.resource::<Town>().builders_busy(), 1);
    for bad in ["load 9000 v1|5|garbage@1", "load 9000 v9|5|500,500,50,0,2;1,0,1,13,13,0,0,0,0@1", "load x"] {
        let mut s = Sim::<Village>::new(Options::default()).unwrap();
        s.post(bad);
        s.run(100.0);
        assert_eq!(s.world().resource::<Town>().buildings.len(), 1, "{bad}");
        assert!(s.world().resource::<Ui>().loaded);
    }
    // Unreadable kept text plays fresh but never saves over it (failed_load_never_uploads).
    for bad in ["load 9000 v1|5|garbage@1", "load 9000 v9|5|500,500,50,0,2;1,0,1,13,13,0,0,0,0@1"] {
        let mut s = Sim::<Village>::new(Options::default()).unwrap();
        s.post(bad);
        s.run(12_000.0);
        assert_eq!(s.world().resource::<Shown>().last_save, "", "{bad}");
    }
}

#[test]
fn decode_rejects_villages_play_could_not_make() {
    let ok = "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,0,0,0,0,0,0";
    assert!(Town::decode(ok).is_some());
    for bad in [
        "500,500,50,0,1;1,0,1,13,13,0,0,0,0",                              // next_id not above ids
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;1,1,1,0,0,0,0,0,0",            // duplicate id
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,0,1,0,0,0,0,0,0",            // two town halls
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,12,12,0,0,0,0",          // overlap
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,28,0,0,0,0,0",           // off the grid
        "500,500,50,0,4;1,0,1,13,13,0,0,0,0;2,1,0,0,0,9,0,0,0;3,2,0,5,0,9,0,0,0", // 2 busy, 1 builder
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,2147483647,0,0,0,0,0",   // huge x
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,99,1,0,0,0,0,0,0",           // unknown kind
        // Over max level, over the mine's cap and over storage are clamped now:
        // see decode_clamps_not_refuses (save.rs).
    ] {
        assert!(Town::decode(bad).is_none(), "{bad}");
    }
}

#[test]
fn a_double_click_on_place_spends_no_gems() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(50.0);
    sim.post("buy goldmine");
    sim.run(50.0);
    sim.post("tap 3 3");
    sim.run(50.0);
    sim.post("place");
    sim.run(50.0);
    // The second click of a double-click lands where Place was.
    sim.post("skip gems");
    sim.run(50.0);
    let w = sim.world();
    assert_eq!(w.resource::<Ui>().selected, 0);
    let town = w.resource::<Town>();
    assert_eq!(town.gems, START_GEMS);
    assert!(town.get(2).unwrap().busy());
}

#[test]
fn picking_follows_the_camera_when_panned_and_zoomed() {
    let view = Vec2::new(1440.0, 756.0);
    let home = tile_at(view, view / 2.0, &CamView::home()).unwrap();
    let panned = CamView { pan_x: 5.0, pan_z: -3.0, ..CamView::home() };
    assert_eq!(tile_at(view, view / 2.0, &panned), Some((home.0 + 5, home.1 - 3)));
    let close = CamView { zoom: ZOOM_MIN, ..CamView::home() };
    assert_eq!(tile_at(view, view / 2.0, &close), Some(home));
    // Zoomed in, the same screen offset covers fewer tiles.
    let off = Vec2::new(900.0, 378.0);
    let far_x = tile_at(view, off, &CamView::home()).unwrap().0 - home.0;
    let near_x = tile_at(view, off, &close).unwrap().0 - home.0;
    assert!(near_x < far_x, "{near_x} < {far_x}");
}

#[test]
fn the_whole_village_fits_the_screen_at_home_zoom() {
    let view = Vec2::new(1440.0, 810.0);
    let v = CamView::home();
    // A point just inside each grid corner is on screen: walk the screen and find all four.
    let mut seen = [false; 4];
    for sy in (0..810).step_by(6) {
        for sx in (0..1440).step_by(6) {
            if let Some((x, z)) = tile_at(view, Vec2::new(sx as f32, sy as f32), &v) {
                for (i, c) in [(0, 0), (29, 0), (0, 29), (29, 29)].iter().enumerate() {
                    if (x, z) == *c {
                        seen[i] = true;
                    }
                }
            }
        }
    }
    assert_eq!(seen, [true; 4]);
}

#[test]
fn a_drag_keeps_the_ground_under_the_finger() {
    let view = Vec2::new(1440.0, 810.0);
    let mut sim: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();
    sim.viewport(view.x, view.y);
    sim.run(100.0);
    let start = Vec2::new(500.0, 300.0);
    // Short enough to stay inside the pan limit.
    let end = Vec2::new(650.0, 380.0);
    let before = ground_at(view, start, &sim.world().resource::<CamView>()).unwrap();
    let mut t = 100.0;
    let send = |sim: &mut Sim<Village>, phase, at: Vec2, d: Vec2, t: f64| {
        sim.input(InputEvent::Pointer {
            id: 1, phase, x: at.x, y: at.y, dx: d.x, dy: d.y, buttons: 1, at_ms: t, player: 0,
        });
    };
    send(&mut sim, PointerPhase::Down, start, Vec2::ZERO, t);
    sim.run(34.0);
    let step = (end - start) / 10.0;
    for i in 1..=10 {
        t += 34.0;
        send(&mut sim, PointerPhase::Move, start + step * i as f32, step, t);
        sim.run(34.0);
    }
    t += 34.0;
    send(&mut sim, PointerPhase::Up, end, Vec2::ZERO, t);
    sim.run(100.0);
    let after = ground_at(view, end, &sim.world().resource::<CamView>()).unwrap();
    let off = (after - before).length();
    assert!(off < 1.0, "the ground slipped {off} m under the finger");
}

/// A slice-1 save, captured before slice 2 changed anything. It must always load.
const V1_SAVE: &str = "v1|200|500,350,50,0,3;1,0,1,13,13,0,0,0,0;2,1,0,9,9,10200,200,0,200";

#[test]
fn a_slice_one_save_still_loads_with_an_empty_army() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.post(format!("load 9000 {V1_SAVE}@9000"));
    sim.run(100.0);
    let town = sim.world().resource::<Town>();
    assert_eq!(town.buildings.len(), 2);
    assert_eq!(town.buildings[1].kind, Kind::GoldMine);
    assert_eq!(town.army, vec![0, 0, 0]);
    assert!(town.queue.is_empty());
}

fn army_town() -> Town {
    let mut t = town_with(1000, 1000);
    let now = 0;
    let b = t.place(Kind::Barracks, 0, 0, now).unwrap();
    t.skip(b, now, Pay::Gems).unwrap();
    let c = t.place(Kind::ArmyCamp, 5, 0, now).unwrap();
    t.skip(c, now, Pay::Gems).unwrap();
    t.elixir = 1000;
    t
}

#[test]
fn new_kinds_go_at_the_end_so_old_saves_keep_their_buildings() {
    assert_eq!(&KINDS[..6], &[
        Kind::TownHall, Kind::GoldMine, Kind::ElixirCollector,
        Kind::GoldStorage, Kind::ElixirStorage, Kind::BuilderHut,
    ]);
}

#[test]
fn training_needs_a_barracks_costs_elixir_and_takes_time() {
    let mut t = town_with(1000, 1000);
    assert_eq!(t.can_train(Troop::Fighter), Err("Build a Barracks".into()));
    let mut t2 = army_town();
    assert_eq!(t2.housing_cap(), 20);
    t2.train(Troop::Fighter, 0).unwrap();
    t2.train(Troop::Fighter, 0).unwrap();
    assert_eq!(t2.elixir, 940);
    assert_eq!(t2.training_left(0), 6);
    t2.settle(5_999);
    assert_eq!(t2.trained(Troop::Fighter), 0);
    t2.settle(6_000);
    assert_eq!(t2.trained(Troop::Fighter), 1);
    t2.settle(12_000);
    assert_eq!(t2.trained(Troop::Fighter), 2);
    assert!(t2.queue.is_empty());
    // Archers train at level 1; Brutes need barracks level 2.
    assert!(t2.can_train(Troop::Archer).is_ok());
    assert_eq!(t2.can_train(Troop::Brute), Err("Needs Barracks level 2".into()));
    t.gold = 0;
}

#[test]
fn housing_counts_queued_troops_and_cancelling_refunds_in_full() {
    let mut t = army_town();
    // The queue holds a whole army: only housing (20) limits it, and queued troops count.
    for _ in 0..20 {
        t.train(Troop::Fighter, 0).unwrap();
    }
    assert_eq!(t.housing_used(), 20);
    assert_eq!(t.can_train(Troop::Fighter), Err("Muster Ground full".into()));
    t.settle(200_000);
    assert_eq!(t.trained(Troop::Fighter), 20);
    assert_eq!(t.can_train(Troop::Fighter), Err("Muster Ground full".into()));
    assert!(QUEUE_MAX as u32 >= Kind::ArmyCamp.housing(MAX_LEVEL) * Kind::ArmyCamp.max_count(MAX_LEVEL) as u32);
    let mut q = army_town();
    q.train(Troop::Fighter, 0).unwrap();
    q.train(Troop::Fighter, 0).unwrap();
    let before = q.elixir;
    q.cancel_training(0, 3_000).unwrap();
    assert_eq!(q.elixir, before + 30);
    assert_eq!(q.training_left(3_000), 6, "the next one starts from the cancel");
}

#[test]
fn training_finishes_while_away() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(50.0);
    {
        let mut town = sim.world_mut().resource_mut::<Town>();
        *town = army_town();
    }
    sim.post("train fighter");
    sim.post("train fighter");
    sim.post("train fighter");
    sim.run(50.0);
    sim.post("away 60");
    sim.run(50.0);
    assert_eq!(sim.world().resource::<Town>().trained(Troop::Fighter), 3);
}

#[test]
fn the_army_round_trips_through_the_save_text() {
    let mut t = army_town();
    t.train(Troop::Fighter, 0).unwrap();
    t.settle(5_000);
    t.train(Troop::Fighter, 5_000).unwrap();
    let back = Town::decode(&t.encode()).unwrap();
    assert_eq!(back, t);
    // Army parts that aren't troops are refused; a missing or extra troop
    // count is padded or truncated, and an army over its housing is trimmed.
    let text = t.encode();
    let (head, _) = text.split_once('#').unwrap();
    for bad in ["#0,0,0,0#7", "#0,x,0,0#", "#0,0,0,0#0,9"] {
        assert!(Town::decode(&format!("{head}{bad}")).is_none(), "{bad}");
    }
    assert_eq!(Town::decode(&format!("{head}#1,2#0")).unwrap().army, vec![1, 0, 0]);
    assert_eq!(Town::decode(&format!("{head}#1,1,1,4,5#0")).unwrap().army, vec![1, 1, 1]);
    let over = Town::decode(&format!("{head}#99,0,0,0#")).unwrap();
    assert_eq!((over.army[0], over.housing_used()), (20, 20));
}

#[test]
fn dev_shortcuts_fill_storage_and_raise_the_town_hall() {
    let mut t = Town::new();
    t.dev_rich();
    assert_eq!((t.gold, t.elixir), t.caps());
    t.dev_town_hall(0);
    assert_eq!(t.town_hall_level(), 2);
    assert_eq!(Kind::ArcherTower.max_count(t.town_hall_level()), 1);
}

// ---- Raids (SPEC-2 G5/G6) ----

fn raid_world() -> Sim<Village> {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(50.0);
    let mut t = army_town();
    t.army = vec![8, 0, 0];
    *sim.world_mut().resource_mut::<Town>() = t;
    sim
}

#[test]
fn a_raid_needs_troops() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.run(50.0);
    command(sim.world_mut(), "attack 0");
    assert!(!fight::active(sim.world()));
    assert_eq!(sim.world().resource::<Ui>().toast, "Train some troops first");
}

#[test]
fn a_whole_raid_takes_troops_brings_loot_home_and_keeps_stars() {
    let mut sim = raid_world();
    let gold_before = sim.world().resource::<Town>().gold;
    command(sim.world_mut(), "attack 0");
    assert!(fight::active(sim.world()));
    // Inside the red border is refused and takes nothing.
    command(sim.world_mut(), "tap 13 13");
    assert_eq!(sim.world().resource::<Town>().army[0], 8);
    for x in 0..8 {
        command(sim.world_mut(), &format!("tap {x} 0"));
    }
    assert_eq!(sim.world().resource::<Town>().army[0], 0, "dropped troops leave the army at once");
    // Village commands do nothing mid-raid.
    let gems = sim.world().resource::<Town>().gems;
    command(sim.world_mut(), "skip gems");
    command(sim.world_mut(), "buy goldmine");
    assert_eq!(sim.world().resource::<Town>().gems, gems);
    assert!(sim.world().resource::<Ui>().placing.is_empty());
    // Fight it out (2.5 minutes at most).
    sim.run(160_000.0);
    let r = sim.world().resource::<fight::Fight>().result.clone().expect("the raid ended");
    assert_eq!((r.stars, r.percent, r.used), (3, 100, 8));
    let town = sim.world().resource::<Town>();
    assert_eq!(town.gold, gold_before + r.gold);
    assert_eq!((town.stars[0], town.looted[0]), (3, 0b11111));
    drop(town);
    let hud = hud(sim.world(), now_ms(sim.world()), String::new());
    assert!(hud.has_result && hud.result_stars == 3);
    command(sim.world_mut(), "home");
    assert!(!fight::active(sim.world()));
    assert_eq!(hud_after(&sim).campaign[0].stars, 3);
}

fn hud_after(sim: &Sim<Village>) -> Hud {
    hud(sim.world(), now_ms(sim.world()), String::new())
}

#[test]
fn surrender_ends_the_raid_with_what_was_won() {
    let mut sim = raid_world();
    command(sim.world_mut(), "attack 1");
    command(sim.world_mut(), "tap 0 0");
    command(sim.world_mut(), "surrender");
    sim.run(100.0);
    let r = sim.world().resource::<fight::Fight>().result.clone().unwrap();
    assert_eq!((r.stars, r.used), (0, 1));
    assert_eq!(sim.world().resource::<Town>().army[0], 7);
}

#[test]
fn time_away_does_not_move_the_raid_clock() {
    let mut sim = raid_world();
    command(sim.world_mut(), "attack 0");
    command(sim.world_mut(), "tap 0 0");
    let before = sim.world().resource::<fight::Fight>().battle.as_ref().unwrap().tick;
    away(sim.world_mut(), 3_600_000);
    let after = sim.world().resource::<fight::Fight>().battle.as_ref().unwrap().tick;
    assert_eq!(before, after);
    // Every published raid number is sane.
    let h = hud_after(&sim);
    assert!(h.raiding && h.raid_secs <= 150 && h.raid_percent <= 100 && h.raid_stars <= 3);
}

#[test]
fn a_save_mid_raid_keeps_the_army_reduced_and_brings_no_loot() {
    let mut sim = raid_world();
    command(sim.world_mut(), "attack 0");
    command(sim.world_mut(), "tap 0 0");
    command(sim.world_mut(), "tap 1 0");
    let text = sim.world().resource::<Town>().encode();
    let back = Town::decode(&text).unwrap();
    assert_eq!(back.army[0], 6);
    assert_eq!(back.stars[0], 0);
}


// ---- Study (SPEC-3 T2) ----

const DAY: u32 = 20_000;
const H: u64 = 3_600_000;

#[test]
fn reviews_pay_knowledge_chests_once_a_day_and_roll_at_4am() {
    let mut t = town_with(0, 0);
    let said = t.add_reviews(49, DAY, 0);
    assert!(said.is_empty());
    assert_eq!((t.knowledge, t.study.toward), (9, 4));
    let said = t.add_reviews(1, DAY, 0);
    assert!(said.iter().any(|s| s.contains("streak")) && said.iter().any(|s| s.contains("Chest 1")), "{said:?}");
    let gold = t.gold;
    assert!(gold > 0);
    // The same tier never pays twice in a day.
    assert!(!t.add_reviews(10, DAY, 0).iter().any(|s| s.contains("Chest 1")));
    // A new day opens the chests again.
    assert!(t.add_reviews(50, DAY + 1, 0).iter().any(|s| s.contains("Chest 1")));
    // A late report for an old day pays only Knowledge.
    let k = t.knowledge;
    assert!(t.add_reviews(500, DAY, 0).is_empty());
    assert_eq!(t.knowledge, k + 100);
    // 4 am local rollover: 03:59 and 04:00 on the same UTC date are different review days.
    let midnight = 1_764_288_000_000u64; // 2025-11-28 00:00 UTC
    assert_eq!(review_day(midnight + 4 * H - 60_000, 0) + 1, review_day(midnight + 4 * H, 0));
    // Offsets move it: 04:00 in UTC-7 is 11:00 UTC.
    assert_eq!(review_day(midnight + 11 * H, -420), review_day(midnight + 4 * H, 0));
}

#[test]
fn streaks_count_days_in_a_row_and_break_after_a_missed_day() {
    let mut t = town_with(0, 0);
    for d in 0..6 {
        t.add_reviews(50, DAY + d, 0);
    }
    assert_eq!(t.study.streak_on(DAY + 5), 6);
    let gems = t.gems;
    let said = t.add_reviews(50, DAY + 6, 0);
    assert_eq!(t.study.streak, 7);
    assert!(said.iter().any(|s| s.contains("Week chest")) && t.gems == gems + 25, "{said:?}");
    // Skipping a day shows the streak as broken, and the next study day starts at 1.
    assert_eq!(t.study.streak_on(DAY + 8), 0);
    t.add_reviews(50, DAY + 8, 0);
    assert_eq!(t.study.streak, 1);
}

#[test]
fn power_hours_double_production_and_capacity_then_expire() {
    let mut t = town_with(1000, 1000);
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.settle(20_000);
    let mark = t.get(id).unwrap().mark;
    // 100 reviews: one power hour starting now.
    t.add_reviews(100, DAY, mark);
    assert!(t.study.powered(mark) && !t.study.powered(mark + H));
    let b = t.get(id).unwrap().clone();
    // Level 1 makes 250/h and holds 2,500: in power it makes 500/h and holds 5,000.
    assert_eq!(b.waiting_in(mark + 30 * MIN, t.power()), 250);
    assert_eq!(b.waiting_in(mark + 20 * HOUR, t.power()), 5000);
    assert_eq!(b.waiting(mark + 20 * HOUR), 2500, "without the window: normal cap");
    // A 4-hour cap per day.
    t.add_reviews(900, DAY, mark);
    assert_eq!(t.study.power_today, 4);
    assert_eq!(t.study.power_until, mark + 4 * H);
}

#[test]
fn training_is_twice_as_fast_in_a_power_hour() {
    let mut t = army_town();
    let normal = t.train_ms(Troop::Fighter, 0);
    t.add_reviews(100, DAY, 0);
    assert_eq!(t.train_ms(Troop::Fighter, 0) * 2, normal);
    assert_eq!(t.train_ms(Troop::Fighter, H), normal);
}

#[test]
fn a_save_without_study_still_loads_and_bad_study_is_refused() {
    let t = town_with(500, 500);
    let text = t.encode();
    let old: Vec<&str> = text.split('#').collect();
    let pre_study = old[..5].join("#");
    assert_eq!(Town::decode(&pre_study).unwrap().study, Study::default());
    assert!(Town::decode(&format!("{pre_study}#1,2,3")).is_none());
    assert!(Town::decode(&format!("{pre_study}#1,2,3,4,5,6,7,8,x")).is_none());
}

#[test]
fn a_power_hour_bank_above_the_normal_cap_survives_save_and_reload() {
    let mut t = town_with(1000, 1000);
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.settle(20_000);
    let mark = t.get(id).unwrap().mark;
    t.add_reviews(100, DAY, mark);
    // A second power hour banks every mine at its doubled cap.
    t.add_reviews(100, DAY, mark + 20 * HOUR);
    assert_eq!(t.get(id).unwrap().banked_milli, 5_000_000);
    let back = Town::decode(&t.encode()).expect("a power-hour bank must reload");
    assert_eq!(back.get(id).unwrap().banked_milli, 5_000_000);
}

// ---- Cloud save (SPEC-3 T3) ----

fn net(sim: &mut Sim<Village>, line: &str) {
    sim.bind(
        &[Value::Number(7.0), Value::Bool(false), Value::Bool(false), Value::Number(1_795_850_000_000.0), Value::str(line)],
        None,
    )
    .unwrap();
    sim.run(100.0);
}

fn ask(sim: &Sim<Village>) -> String {
    sim.world().resource::<village_logic::cloud::Cloud>().ask.clone()
}

#[test]
fn cloud_joins_signs_in_counts_anki_reviews_once_and_uploads_with_its_version() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    assert!(ask(&sim).starts_with("1\tjoin"), "{:?}", ask(&sim));
    net(&mut sim, "1\tok\tp@players.invalid\tsecret");
    assert_eq!(ask(&sim), "2\tsignin\t\tp@players.invalid\tsecret");
    net(&mut sim, "2\tok\tTOKEN");
    assert!(ask(&sim).starts_with("3\tstate\tTOKEN"));
    // No cloud save yet, and 60 Anki reviews waiting (event 5).
    let k = sim.world().resource::<Town>().knowledge;
    net(&mut sim, "3\tok\t0\t5:60:20000\t");
    assert_eq!(sim.world().resource::<Town>().knowledge, k + 12);
    assert_eq!(sim.world().resource::<Town>().study.last_event, 5);
    // The same event again (say after a reload before the server heard) counts nothing.
    sim.run(6000.0);
    let up = ask(&sim);
    assert!(up.starts_with("4\tsave\tTOKEN\t0\t5\t5\tv2|"), "{up:?}");
    net(&mut sim, "4\tok\t1");
    {
        let c = sim.world().resource::<village_logic::cloud::Cloud>();
        assert_eq!((c.version, c.applied.len()), (1, 0));
    }
    // Nothing changed: nothing more goes up.
    sim.run(12_000.0);
    assert_eq!(ask(&sim), "");
}

#[test]
fn a_cloud_conflict_loads_the_other_devices_save() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    assert!(ask(&sim).starts_with("1\tsignin"));
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t0\t\t");
    sim.post("devmode on");
    sim.post("dev rich");
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("3\tsave\tTOKEN\t0\t"));
    // Another device saved a village with 777 gold first.
    let mut other = Town::new();
    other.gold = 777;
    let text = format!("v2|5000|{}@1795850001000", other.encode());
    net(&mut sim, &format!("3\tconflict\t4\t{text}"));
    // A conflict fetches the cloud save again and takes it, whatever its time.
    assert!(ask(&sim).starts_with("4\tstate\tTOKEN"), "{:?}", ask(&sim));
    net(&mut sim, &format!("4\tok\t4\t\t{text}"));
    assert_eq!(sim.world().resource::<Town>().gold, 777);
    assert_eq!(sim.world().resource::<village_logic::cloud::Cloud>().version, 4);
}

#[test]
fn a_conflict_during_a_raid_waits_and_never_overwrites_the_newer_save() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t0\t\t");
    let mut t = army_town();
    t.army = vec![8, 0, 0];
    *sim.world_mut().resource_mut::<Town>() = t;
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("3\tsave"));
    sim.post("attack 0");
    sim.run(100.0);
    net(&mut sim, "3\tconflict\t9\tv2|1|x@1");
    // Mid-raid: no fetch, and above all no save over version 9.
    sim.run(20_000.0);
    assert_eq!(ask(&sim), "");
    assert!(village_logic::fight::active(sim.world()));
    sim.post("surrender");
    sim.run(200.0);
    sim.post("home");
    sim.run(200.0);
    assert!(ask(&sim).starts_with("4\tstate"), "{:?}", ask(&sim));
}

#[test]
fn anki_reviews_lost_to_a_conflict_are_counted_from_the_winning_save() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    let k = sim.world().resource::<Town>().knowledge;
    net(&mut sim, "2\tok\t3\t5:50:20000\t");
    assert_eq!(sim.world().resource::<Town>().knowledge, k + 10);
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("3\tsave\tTOKEN\t3\t5\t5\t"), "{:?}", ask(&sim));
    // Another device won with a save that never counted event 5.
    let other = Town::new();
    let text = format!("v2|5000|{}@1795850001000", other.encode());
    net(&mut sim, &format!("3\tconflict\t4\t{text}"));
    net(&mut sim, &format!("4\tok\t4\t5:50:20000\t{text}"));
    assert_eq!(sim.world().resource::<Town>().knowledge, 10, "event 5 counted into the adopted save");
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("5\tsave\tTOKEN\t4\t5\t5\t"), "{:?}", ask(&sim));
}

#[test]
fn offline_keeps_playing_and_retries() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\toffline");
    assert_eq!(ask(&sim), "");
    assert!(sim.world().resource::<village_logic::cloud::Cloud>().status.contains("offline"));
    sim.run(16_000.0);
    assert!(ask(&sim).starts_with("2\tjoin"));
}

#[test]
fn signing_in_again_after_the_token_expires_keeps_newer_local_play() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t0\t\t");
    sim.post("devmode on");
    sim.post("dev rich");
    sim.run(6000.0);
    let up = ask(&sim);
    let sent_text = up.rsplit('\t').next().unwrap().to_string();
    net(&mut sim, "3\tok\t1");
    // More play, then the token expires on the next upload.
    sim.post("dev army");
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("4\tsave"));
    net(&mut sim, "4\tauth");
    net(&mut sim, "5\tok\tTOKEN2");
    // The cloud still holds this device's own upload (version 1): keep playing on the newer local village.
    let army_before = sim.world().resource::<Town>().army.clone();
    net(&mut sim, &format!("6\tok\t1\t\t{sent_text}"));
    assert_eq!(sim.world().resource::<Town>().army, army_before);
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("7\tsave\tTOKEN2\t1\t"), "{:?}", ask(&sim));
}

#[test]
fn a_restore_code_moves_the_village_to_this_device() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred new@players.invalid pw");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t0\t\t");
    // Bad codes never leave the device.
    sim.post("restore AB");
    sim.run(100.0);
    assert_eq!(ask(&sim), "");
    sim.post("restore abcd-2345");
    sim.run(100.0);
    assert_eq!(ask(&sim), "3\tclaim\t\tABCD2345");
    net(&mut sim, "3\terror\tno account has that code");
    assert!(sim.world().resource::<Ui>().toast.contains("no account has that code"));
    sim.post("restore ABCD2345");
    sim.run(100.0);
    net(&mut sim, "4\tok\told@players.invalid\tNEW/pw=");
    assert_eq!(ask(&sim), "5\tsignin\t\told@players.invalid\tNEW/pw=");
    net(&mut sim, "5\tok\tTOKEN2");
    // The other device's village (older than this one) still wins.
    let mut other = Town::new();
    other.gold = 888;
    let text = format!("v2|5000|{}@1000", other.encode());
    net(&mut sim, &format!("6\tok\t7\t\t{text}"));
    assert_eq!(sim.world().resource::<Town>().gold, 888);
    assert_eq!(sim.world().resource::<village_logic::cloud::Cloud>().version, 7);
}

#[test]
fn the_restore_code_shows_once_made() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid pw");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    sim.post("restorecode");
    net(&mut sim, "1\tok\tTOKEN");
    assert_eq!(ask(&sim), "2\tmkcode\tTOKEN");
    net(&mut sim, "2\tok\tQRST6789");
    assert_eq!(sim.world().resource::<village_logic::cloud::Cloud>().code, "QRST6789");
    assert_eq!(hud_after(&sim).restore_code, "QRST6789");
    assert!(ask(&sim).starts_with("3\tstate"));
}

#[test]
fn a_device_whose_village_moved_stops_saving_and_never_makes_a_copy() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred old@players.invalid pw");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t3\t\t");
    sim.post("devmode on");
    sim.post("dev rich");
    sim.run(6000.0);
    assert!(ask(&sim).starts_with("3\tsave"));
    // The server refuses the save (moved), and the old password no longer signs in.
    net(&mut sim, "3\tauth");
    assert!(ask(&sim).starts_with("4\tsignin"));
    net(&mut sim, "4\tauth");
    sim.run(30_000.0);
    assert_eq!(ask(&sim), "", "no join, no upload");
    assert!(sim.world().resource::<village_logic::cloud::Cloud>().status.contains("moved"));
    // A code can still bring a village here.
    sim.post("restore ABCD2345");
    sim.run(100.0);
    assert!(ask(&sim).starts_with("5\tclaim"));
}

#[test]
fn a_bug_report_goes_up_with_the_save_and_waits_for_sign_in() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid pw");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    sim.post("bug the\tshop froze");
    net(&mut sim, "1\tok\tTOKEN");
    let up = ask(&sim);
    assert!(up.starts_with("2\tbug\tTOKEN\tthe shop froze\tv2|"), "{up:?}");
    net(&mut sim, "2\toffline");
    sim.run(16_000.0);
    assert!(ask(&sim).starts_with("3\tbug\tTOKEN\tthe shop froze"));
    net(&mut sim, "3\tok");
    assert_eq!(sim.world().resource::<Ui>().toast, "Thanks! Bug report sent");
}
