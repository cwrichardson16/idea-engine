use exact_game::*;
use village_logic::economy::*;
use village_logic::*;

const S: u64 = 1000;

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
    assert_eq!(t.can_buy(Kind::GoldMine).unwrap_err(), "Need 150 elixir");
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
    assert_eq!(t.can_buy(Kind::GoldMine).unwrap_err(), "Upgrade Town Hall");
    assert_eq!(t.can_buy(Kind::BuilderHut).unwrap_err(), "Max built");
    assert_eq!(t.can_buy(Kind::TownHall).unwrap_err(), "Max built");
}

// ---- F3: production and collecting ----

#[test]
fn mines_produce_at_their_rate_up_to_their_cap() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    // Built at 10 s; makes 2 gold/s from then; holds 100.
    t.settle(10 * S);
    assert_eq!(t.get(id).unwrap().waiting(10 * S), 0);
    assert_eq!(t.get(id).unwrap().waiting(15 * S), 10);
    assert_eq!(t.get(id).unwrap().waiting(15 * S + 499), 10);
    assert_eq!(t.get(id).unwrap().waiting(10 * S + 1000 * S), 100);
}

#[test]
fn collecting_moves_into_storage_and_overflow_stays() {
    let mut t = Town::new();
    let id = t.place(Kind::GoldMine, 0, 0, 0).unwrap();
    t.gold = 950; // cap 1000
    let got = t.collect(id, 100 * S); // 90 s of 2/s = 180, capped at 100
    assert_eq!(got, 50);
    assert_eq!(t.gold, 1000);
    assert_eq!(t.get(id).unwrap().waiting(100 * S), 50);
    t.gold = 0;
    assert_eq!(t.collect(id, 100 * S), 50);
    assert_eq!(t.get(id).unwrap().waiting(100 * S), 0);
}

#[test]
fn collecting_often_loses_nothing() {
    let mut a = Town::new();
    let mut b = Town::new();
    let ia = a.place(Kind::GoldMine, 0, 0, 0).unwrap();
    let ib = b.place(Kind::GoldMine, 0, 0, 0).unwrap();
    a.gold = 0;
    b.gold = 0;
    // a collects every 333 ms; b once at the end.
    let mut t = 10 * S;
    while t < 40 * S {
        a.collect(ia, t);
        t += 333;
    }
    a.collect(ia, 40 * S);
    b.collect(ib, 40 * S);
    assert_eq!(a.gold, 60);
    assert_eq!(b.gold, 60);
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
    assert_eq!(t.can_buy(Kind::ElixirCollector).unwrap_err(), "No free builder");
    // The hut is instant, needs no builder, and adds one.
    t.place(Kind::BuilderHut, 5, 0, 0).unwrap();
    assert_eq!((t.builders(), t.builders_free()), (2, 1));
    t.place(Kind::ElixirCollector, 8, 0, 0).unwrap();
    assert_eq!(t.builders_free(), 0);
    let th = t.buildings[0].id;
    assert_eq!(t.can_upgrade(th).unwrap_err(), "No free builder");
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
    assert_eq!(t.can_upgrade(id).unwrap_err(), "Upgrade Town Hall first");
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
    // It was full (100) when the upgrade started and kept that; level 2 adds 3/s.
    assert_eq!(b.waiting(110 * S), 130);
}

#[test]
fn nothing_is_made_while_upgrading_but_banked_stays() {
    let mut t = town_with(3000, 3000);
    let th = t.buildings[0].id;
    t.upgrade(th, 0).unwrap();
    t.settle(60 * S);
    let id = t.place(Kind::GoldMine, 0, 0, 60 * S).unwrap();
    t.settle(70 * S);
    // 20 gold made by 80 s, then upgrading for 30 s.
    t.upgrade(id, 80 * S).unwrap();
    assert_eq!(t.get(id).unwrap().waiting(100 * S), 20);
    t.settle(110 * S);
    assert_eq!(t.get(id).unwrap().waiting(110 * S), 20);
    assert_eq!(t.get(id).unwrap().waiting(120 * S), 50);
}

#[test]
fn max_level_stops_upgrades() {
    let mut t = town_with(100_000, 100_000);
    t.elixir = 100_000;
    let th = t.buildings[0].id;
    let mut now = 0;
    for _ in 0..2 {
        t.upgrade(th, now).unwrap();
        now += 200 * S;
        t.settle(now);
        // Storage caps stop gold from going above the cap; refill for the test.
        t.gold = 100_000;
    }
    assert_eq!(t.town_hall_level(), 3);
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
    sim.post("away 60");
    sim.run(100.0);
    let w = sim.world();
    let b = w.resource::<Town>().get(id).unwrap().clone();
    assert_eq!(b.level, 1);
    assert!(b.waiting(now_ms(w)) >= 100);
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
    t.review();
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
    assert_eq!(t.skip_cost(th, 0, Pay::Knowledge), Some(3));
    t.skip(th, 1000, Pay::Gems).unwrap();
    assert_eq!((t.gems, t.town_hall_level()), (44, 2));
}

#[test]
fn knowledge_only_comes_from_reviews() {
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    sim.post("away 3600");
    sim.run(100.0);
    assert_eq!(sim.world().resource::<Town>().knowledge, 0);
    sim.post("review");
    sim.post("review");
    sim.run(100.0);
    assert_eq!(sim.world().resource::<Town>().knowledge, 2);
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
    t.review();
    let text = t.encode();
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
    // The page keeps it stamped with the real time; come back 10 minutes later.
    let kept = format!("{}@5000000", &saved["save ".len()..]);
    let mut again = Sim::<Village>::new(Options::default()).unwrap();
    again.post(format!("load 5600000 {kept}"));
    again.run(100.0);
    let w = again.world();
    let b = w.resource::<Town>().get(id).unwrap().clone();
    assert_eq!(b.level, 1, "the 10 s build finished while away");
    assert_eq!(b.waiting(now_ms(w)), 100, "and the mine filled up");
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
        "500,500,50,0,3;1,0,3,13,13,9,0,0,0",                              // busy past max level
        "500,500,50,0,4;1,0,1,13,13,0,0,0,0;2,1,0,0,0,9,0,0,0;3,2,0,5,0,9,0,0,0", // 2 busy, 1 builder
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,0,0,0,0,999999999,0",    // over the mine's cap
        "99999,500,50,0,2;1,0,1,13,13,0,0,0,0",                            // over storage
        "500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,2147483647,0,0,0,0,0",   // huge x
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
    assert_eq!(t2.housing_cap(), 24);
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
    for _ in 0..10 {
        t.train(Troop::Fighter, 0).unwrap();
    }
    assert_eq!(t.can_train(Troop::Fighter), Err("Queue full".into()));
    t.settle(60_000);
    assert_eq!(t.trained(Troop::Fighter), 10);
    for _ in 0..10 {
        t.train(Troop::Fighter, 60_000).unwrap();
    }
    t.settle(200_000);
    for _ in 0..4 {
        t.train(Troop::Fighter, 200_000).unwrap();
    }
    assert_eq!(t.housing_used(), 24);
    t.settle(400_000);
    assert_eq!(t.can_train(Troop::Fighter), Err("Army camps full".into()));
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
    // Damaged army parts are refused.
    let text = t.encode();
    let (head, _) = text.split_once('#').unwrap();
    for bad in ["#1,2#0", "#1,2,3,4,5#0", "#0,0,0,0#7", "#99,0,0,0#"] {
        assert!(Town::decode(&format!("{head}{bad}")).is_none(), "{bad}");
    }
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
