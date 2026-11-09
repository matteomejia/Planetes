use crate::state::State;
use planetes_core::{EpicIdentity, ScanReport, SourceGame, StoreId};

type Scene = (&'static str, fn(&mut State));
pub(crate) const SCENES: &[Scene] = &[
    ("library", library),
    ("no-art", no_art),
    ("empty", empty),
    ("loading", loading),
    ("error", error),
];
pub(crate) fn library(s: &mut State) {
    s.handle = None;
    let names = [
        "Baldur's Gate 3",
        "Balatro",
        "Control",
        "Cyberpunk 2077",
        "Elden Ring",
        "Fortnite",
        "Hades II",
        "Hollow Knight",
        "Sea of Thieves",
        "Stardew Valley",
    ];
    let games = names
        .iter()
        .enumerate()
        .map(|(i, name)| SourceGame {
            store: if i == 5 {
                StoreId::Epic
            } else {
                StoreId::Steam
            },
            source_id: if i == 5 {
                "Fortnite".into()
            } else {
                (i + 100).to_string()
            },
            title: (*name).into(),
            install_dir: "fixture-game".into(),
            manifest_path: "fixture-manifest".into(),
            icon_path: fixture_art(i, "icon.exe"),
            // A mix of recent, old and never-played games, as a real library has.
            last_played: [
                Some(2),
                Some(0),
                None,
                Some(40),
                Some(9),
                Some(1),
                None,
                Some(400),
                None,
                Some(120),
            ][i]
                .map(|days: u64| crate::history::now() - days * 24 * 60 * 60),
            playtime_minutes: [
                Some(5_820),
                Some(745),
                None,
                Some(4_829),
                Some(10_943),
                None,
                None,
                Some(1_188),
                None,
                Some(42),
            ][i],
            // Screenshots can borrow real art: `PLANETES_FIXTURE_ART` names a folder holding
            // `<row>-hero.jpg`, `<row>-logo.png` and `<row>-icon.exe`. Tests and scenes run fine without it.
            hero_path: fixture_art(i, "hero.jpg"),
            logo_path: fixture_art(i, "logo.png"),
            hero_url: None,
            logo_url: None,
            epic_identity: (i == 5).then(|| EpicIdentity {
                namespace: "fn".into(),
                item_id: "fixture-item".into(),
            }),
        })
        .collect();
    s.apply_report(ScanReport {
        games,
        warnings: Vec::new(),
    });
}
fn fixture_art(row: usize, name: &str) -> Option<std::path::PathBuf> {
    let path = std::path::Path::new(&std::env::var_os("PLANETES_FIXTURE_ART")?)
        .join(format!("{row}-{name}"));
    path.is_file().then_some(path)
}
/// A game without local art (Epic): the panel's icon fallback.
fn no_art(s: &mut State) {
    library(s);
    s.query.text = "fortnite".into();
    s.filter();
}
fn empty(s: &mut State) {
    s.handle = None;
    s.apply_report(ScanReport::default());
}
fn loading(s: &mut State) {
    *s = State::default();
}
fn error(s: &mut State) {
    library(s);
    s.error = Some("The store launcher is unavailable. Install or repair it and try again.".into());
}
