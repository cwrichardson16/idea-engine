//! Save safety (ROADMAP.md step 1): every past save format still loads, rule
//! retunes clamp instead of refusing, and a save that can't be read is never
//! overwritten, locally or in the cloud.
use exact_game::*;
use village_logic::cloud::Cloud;
use village_logic::economy::*;
use village_logic::*;

/// One frozen save string per format ever shipped, with the buildings'
/// (kind, x, z) it must decode to. Never edit one: add the next format's
/// string when SAVE_CONTRACT is bumped (village/CLAUDE.md "SAVE_CONTRACT").
const FROZEN: &[(&str, &str, &[(Kind, i32, i32)])] = &[
    (
        "v1: slice 1, no '#' sections",
        "500,350,50,0,3;1,0,1,13,13,0,0,0,0;2,1,0,9,9,10200,200,0,200",
        &[(Kind::TownHall, 13, 13), (Kind::GoldMine, 9, 9)],
    ),
    (
        "army: + #army counts,queue_start#queue",
        "420,380,48,0,4;1,0,1,13,13,0,0,0,0;2,6,1,0,0,0,0,0,25000;3,7,1,5,0,0,0,0,30000#3,2,0,61000#0,1",
        &[(Kind::TownHall, 13, 13), (Kind::Barracks, 0, 0), (Kind::ArmyCamp, 5, 0)],
    ),
    (
        "campaign: + #stars#looted",
        "900,640,45,0,5;1,0,2,13,13,0,0,0,0;2,6,1,0,0,0,0,0,25000;3,7,1,5,0,0,0,0,30000;4,1,2,9,9,0,0,100000,90000#0,0,0,0#\
         #3,1,0,0#31,5,0,0",
        &[(Kind::TownHall, 13, 13), (Kind::Barracks, 0, 0), (Kind::ArmyCamp, 5, 0), (Kind::GoldMine, 9, 9)],
    ),
    (
        "study: + #9 study fields (no last_event)",
        "900,640,45,12,5;1,0,2,13,13,0,0,0,0;2,6,1,0,0,0,0,0,25000;3,7,1,5,0,0,0,0,30000;4,1,2,9,9,0,0,200000,90000#0,0,0,0#\
         #3,1,0,0#31,5,0,0#20000,60,1,0,0,1,20000,0,0",
        &[(Kind::TownHall, 13, 13), (Kind::Barracks, 0, 0), (Kind::ArmyCamp, 5, 0), (Kind::GoldMine, 9, 9)],
    ),
    (
        "v4: the 10-field study format, the last before SAVE_CONTRACT was written in",
        "1900,1500,40,52,6;1,0,2,13,13,0,0,0,0;2,6,2,0,0,0,0,0,25000;3,7,2,5,0,0,0,0,30000;4,1,2,9,9,0,0,400000,90000;\
         5,3,2,18,9,0,0,0,40000#4,3,1,61000#0,0#3,2,0,0#31,7,0,0#20000,260,3,2,0,1,20000,72000000,79200000,5",
        &[
            (Kind::TownHall, 13, 13),
            (Kind::Barracks, 0, 0),
            (Kind::ArmyCamp, 5, 0),
            (Kind::GoldMine, 9, 9),
            (Kind::GoldStorage, 18, 9),
        ],
    ),
    (
        "v5: step 1, + #contract,camp_day",
        "1900,1500,40,52,6;1,0,2,13,13,0,0,0,0;2,6,2,0,0,0,0,0,25000;3,7,2,5,0,0,0,0,30000;4,1,2,9,9,0,0,400000,90000;\
         5,3,2,18,9,0,0,0,40000#4,3,1,61000#0,0#3,2,0,0#31,7,0,0#20000,260,3,2,0,1,20000,72000000,79200000,5#5,20001",
        &[
            (Kind::TownHall, 13, 13),
            (Kind::Barracks, 0, 0),
            (Kind::ArmyCamp, 5, 0),
            (Kind::GoldMine, 9, 9),
            (Kind::GoldStorage, 18, 9),
        ],
    ),
];

#[test]
fn old_saves_decode() {
    for (name, text, want) in FROZEN {
        let town = Town::decode(text).unwrap_or_else(|| panic!("{name} no longer decodes"));
        let got: Vec<(Kind, i32, i32)> = town.buildings.iter().map(|b| (b.kind, b.x, b.z)).collect();
        assert_eq!(&got, want, "{name}");
        // What decodes re-encodes in today's format and reads back the same.
        assert!(town.encode().ends_with(&format!("#{SAVE_CONTRACT},{}", town.camp_day)), "{name}");
        assert_eq!(Town::decode(&town.encode()).as_ref(), Some(&town), "{name}");
    }
    // The newest frozen string is today's format.
    let newest = FROZEN.last().unwrap().1;
    assert_eq!(Town::decode(newest).unwrap().encode(), newest, "add a frozen string for SAVE_CONTRACT {SAVE_CONTRACT}");
    // Old details survive: the army-era queue, campaign stars, study counts.
    let army = Town::decode(FROZEN[1].1).unwrap();
    assert_eq!((army.army.clone(), army.queue.clone()), (vec![3, 2, 0], vec![Troop::Fighter, Troop::Archer]));
    let v4 = Town::decode(FROZEN[4].1).unwrap();
    assert_eq!((v4.stars.clone(), v4.study.reviews, v4.study.last_event, v4.camp_day), (vec![3, 2, 0, 0], 260, 5, 0));
    assert_eq!(Town::decode(FROZEN[5].1).unwrap().camp_day, 20001);
    // And through the page's load path, every one loads into the world.
    for (name, text, want) in FROZEN {
        let mut sim = Sim::<Village>::new(Options::default()).unwrap();
        sim.post(format!("load 9000 v2|5000|{text}@9000"));
        sim.run(100.0);
        let w = sim.world();
        assert!(!w.resource::<Ui>().load_failed, "{name}");
        assert_eq!(w.resource::<Town>().buildings.len(), want.len(), "{name}");
    }
}

#[test]
fn decode_clamps_not_refuses() {
    // Gold and grain over the cap, a mine banked over its holds, a Keep over
    // max level, a Mason's Lodge over its max level, a missing troop count, an
    // extra camp star and a missing looted mask.
    let a = "999999,999999,50,0,6;1,0,9,13,13,0,0,0,0;2,1,1,0,0,0,0,999999999,0;3,7,1,5,0,0,0,0,0;\
             4,6,1,10,0,0,0,0,0;5,5,5,20,0,0,0,0,0#3,0,6000#0#3,1,0,0,2#1,2,3#20000,0,0,0,0,0,0,0,0,0#5,20000";
    let t = Town::decode(a).expect("clamped, not refused");
    assert_eq!(t.town_hall_level(), MAX_LEVEL);
    assert_eq!(t.get(5).unwrap().level, Kind::BuilderHut.max_level());
    let (gc, ec) = t.caps();
    assert_eq!((t.gold, t.elixir), (gc, ec));
    let (_, _, holds) = Kind::GoldMine.production(1).unwrap();
    assert_eq!(t.get(2).unwrap().banked_milli, holds * 2000, "a power hour's doubled holds at most");
    assert_eq!(t.army, vec![3, 0, 0]);
    assert_eq!(t.queue, vec![Troop::Fighter]);
    assert_eq!(t.stars, vec![3, 1, 0, 0]);
    assert_eq!(t.looted, vec![1, 2, 3, 0]);
    // Unversioned (v4): an extra troop count, a missing star, an extra looted
    // mask, a Barracks above the Keep, a busy Keep upgrading past max, an army
    // over its housing, stars over 3.
    let b = "500,500,50,0,4;1,0,1,13,13,0,0,0,0;2,7,1,5,0,0,0,0,0;3,6,2,10,0,0,0,0,0#30,1,1,9,0#1#7#7,7,7,7,7";
    let t = Town::decode(b).expect("clamped, not refused");
    assert_eq!(t.get(3).unwrap().level, 1, "no higher than the Keep");
    assert_eq!(t.housing_used(), t.housing_cap());
    assert_eq!(t.army.len(), TROOPS.len());
    assert!(t.queue.is_empty(), "queued troops go first when over housing");
    assert_eq!(t.stars, vec![3, 0, 0, 0]);
    assert_eq!(t.looted, vec![7, 7, 7, 7]);
    let busy_max = "500,500,50,0,2;1,0,8,13,13,99000,1000,0,0";
    let t = Town::decode(busy_max).expect("clamped, not refused");
    assert_eq!((t.town_hall_level(), t.buildings[0].busy()), (MAX_LEVEL, false));
    // Text that isn't a village is still refused.
    for bad in [
        "",
        "garbage",
        "1,2,3",
        "500,500,50,0,2;1,0,1,13,13,0,0,0,0#0,0,0,0",       // 2 sections
        "500,500,50,0,2;1,0,1,13,13,0,0,0,0#0,0,0,0##0,0,0,0", // 4 sections
        "500,500,50,0,2;1,0,1,13,13,0,0,x,0",
        "500,500,50,0,2;1,0,1,13,13,0,0,0,0#0,0,0,0##0,0,0,0#0,0,0,0#1,2,3,4,5,6,7,8,9#6,0", // a newer contract
        "500,500,50,0,2;1,0,1,13,13,0,0,0,0#0,0,0,0##0,0,0,0#0,0,0,0#1,2,3,4,5,6,7,8,9#5", // contract without camp_day
    ] {
        assert!(Town::decode(bad).is_none(), "{bad:?}");
    }
}

fn net(sim: &mut Sim<Village>, line: &str) {
    sim.bind(
        &[Value::Number(7.0), Value::Bool(false), Value::Bool(false), Value::Number(1_795_850_000_000.0), Value::str(line)],
        None,
    )
    .unwrap();
    sim.run(100.0);
}

fn ask(sim: &Sim<Village>) -> String {
    sim.world().resource::<Cloud>().ask.clone()
}

/// Ten simulated minutes in 10 s steps: no upload ever queued, and no save or
/// kept message sent to the page (which is what would change its stored text).
fn ten_minutes_without_a_save(sim: &mut Sim<Village>) {
    for _ in 0..60 {
        sim.run(10_000.0);
        assert!(!ask(sim).contains("\tsave\t"), "a put_save went out: {:?}", ask(sim));
        for m in sim.take_messages() {
            assert!(!m.starts_with("save ") && !m.starts_with("kept "), "the page was told to store {m:?}");
        }
        let w = sim.world();
        assert!(w.resource::<Cloud>().dirty.is_empty());
        assert_eq!(w.resource::<Shown>().last_save, "");
    }
}

#[test]
fn failed_load_never_uploads() {
    // The page kept a save this build can't read (overlapping buildings and a broken army).
    let corrupt = "v2|5000|500,500,50,0,3;1,0,1,13,13,0,0,0,0;2,1,1,13,13,0,0,0,0#x#@1795849000000";
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post(format!("load 1795850000000 {corrupt}"));
    sim.run(100.0);
    sim.take_messages();
    // An error is shown, and stays shown.
    let w = sim.world();
    assert!(w.resource::<Ui>().load_failed);
    assert!(w.resource::<Ui>().toast.contains("couldn't be read"));
    assert_eq!(hud(w, now_ms(w), String::new()).cloud, LOAD_FAILED);
    // The cloud signs in and looks for a readable copy: there is none, but
    // Anki reviews are waiting.
    assert!(ask(&sim).starts_with("1\tsignin"), "{:?}", ask(&sim));
    net(&mut sim, "1\tok\tTOKEN");
    assert!(ask(&sim).starts_with("2\tstate\tTOKEN"));
    net(&mut sim, "2\tok\t0\t5:60:20000\t");
    // Playing on the fresh village changes things, and still nothing is saved.
    sim.post("devmode on");
    sim.post("dev rich");
    sim.post("buy goldmine");
    sim.run(100.0);
    sim.post("place");
    ten_minutes_without_a_save(&mut sim);
    let w = sim.world();
    assert_eq!(hud(w, now_ms(w), String::new()).cloud, LOAD_FAILED);

    // A readable cloud save, if there is one, replaces the fresh village and
    // saving resumes (the cloud copy is the newest readable truth).
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post(format!("load 1795850000000 {corrupt}"));
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    let mut other = Town::new();
    other.gold = 777;
    let good = format!("v2|5000|{}@1795840000000", other.encode());
    net(&mut sim, &format!("2\tok\t3\t\t{good}"));
    assert!(!sim.world().resource::<Ui>().load_failed);
    assert_eq!(sim.world().resource::<Town>().gold, 777);

    // A cloud save this build can't read (say a newer version's) is never
    // uploaded over, even when this device's village is newer.
    let mut sim = Sim::<Village>::new(Options::default()).unwrap();
    net(&mut sim, "");
    sim.post("cred p@players.invalid secret");
    sim.post("load 1795850000000 ");
    sim.run(100.0);
    net(&mut sim, "1\tok\tTOKEN");
    net(&mut sim, "2\tok\t9\t\tv3|5000|from-a-newer-build@1795800000000");
    sim.post("devmode on");
    sim.post("dev rich");
    for _ in 0..60 {
        sim.run(10_000.0);
        assert!(!ask(&sim).contains("\tsave\t"), "uploaded over an unreadable cloud save");
    }
    assert!(sim.world().resource::<Cloud>().status.contains("not uploading"));
}
