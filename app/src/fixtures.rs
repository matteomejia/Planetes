use crate::state::State;
use planetes_core::{EpicIdentity, ScanReport, SourceGame, StoreId};

type Scene = (&'static str, fn(&mut State));
pub(crate) const SCENES: &[Scene] = &[
    ("library", library),
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
            icon_path: None,
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
