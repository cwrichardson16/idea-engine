//! Art budget (ROADMAP.md step 2): what setup waits for stays small enough for
//! a phone's first play, and animated troops ship only the clips they use.
use exact_game::*;
use std::io::Write;
use std::process::{Command, Stdio};
use village_logic::*;

/// Gzip size at the default level (what a static host serves), via the
/// system `gzip`, so the test needs no compression crate.
fn gzip_len(bytes: &[u8]) -> usize {
    let mut child = Command::new("gzip")
        .args(["-c", "-6", "-n"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("asset_budget needs `gzip` on PATH");
    let mut stdin = child.stdin.take().unwrap();
    let input = bytes.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&input).unwrap());
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(out.status.success(), "gzip failed");
    out.stdout.len()
}

const BUDGET: usize = 2_000_000;
const MAX_CLIPS: usize = 5;

#[test]
fn asset_budget() {
    let art = exact_game_bake::game_art(env!("CARGO_MANIFEST_DIR")).unwrap();
    let sim: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();
    let models: std::collections::BTreeMap<&str, &asset::Model> = sim.presentation_models().collect();
    let mut total = 0;
    let mut report = Vec::new();
    for &name in Village::ASSETS {
        let bytes = art.get(name).unwrap_or_else(|| panic!("{name} is in ASSETS but the bake made no such file"));
        let mut files = vec![(name.to_string(), gzip_len(bytes))];
        let model = models.get(name).unwrap_or_else(|| panic!("{name} did not load"));
        // A device fetches one family of each texture: count the largest.
        for tex in &model.textures {
            let stem = tex.strip_suffix(".tex").unwrap();
            let worst = [tex.clone(), format!("{stem}.bc.tex"), format!("{stem}.astc.tex")]
                .into_iter()
                .filter_map(|f| art.get(&f).map(|b| (f, gzip_len(b))))
                .max_by_key(|(_, n)| *n)
                .unwrap_or_else(|| panic!("{name} samples {tex}, which the bake did not make"));
            files.push(worst);
        }
        for (file, n) in files {
            total += n;
            report.push(format!("{file}: {n} B gzip"));
        }
        // Animated models: the stripped clip set only.
        if !model.skins.is_empty() || !model.clips.is_empty() {
            let clips: Vec<&str> = model.clips.iter().map(|c| c.name.as_str()).collect();
            assert!(clips.len() <= MAX_CLIPS, "{name} ships {} clips: {clips:?}", clips.len());
        }
    }
    println!("ASSETS gzip total {total} B\n{}", report.join("\n"));
    assert!(total <= BUDGET, "ASSETS are {total} B gzip, over {BUDGET}:\n{}", report.join("\n"));
}

#[test]
fn the_spearman_has_every_clip_it_plays() {
    let sim: Sim<Village> = exact_game::sim_with_art!(Options::default()).unwrap();
    let (_, knight) = sim
        .presentation_models()
        .find(|(n, _)| *n == fight::SPEARMAN_MODEL)
        .expect("the Spearman's model loads before setup");
    let mut clips: Vec<&str> = knight.clips.iter().map(|c| c.name.as_str()).collect();
    clips.sort();
    let mut want = KNIGHT_CLIPS.to_vec();
    want.sort();
    assert_eq!(clips, want);
    assert!(Village::ASSETS.contains(&fight::SPEARMAN_MODEL), "an animated model must be in ASSETS");
}
