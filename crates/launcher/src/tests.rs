use super::*;
use planetes_core::EpicIdentity;

fn game(store: StoreId, id: &str) -> SourceGame {
    SourceGame {
        store,
        source_id: id.into(),
        title: "Fixture".into(),
        install_dir: "missing-fixture-directory".into(),
        manifest_path: "fixture".into(),
        epic_identity: None,
        icon_path: None,
    }
}

#[test]
fn steam_uses_the_numeric_game_uri_and_rejects_untrusted_input() {
    assert_eq!(
        LaunchPlan::for_game(&game(StoreId::Steam, "730"))
            .unwrap()
            .uri(),
        "steam://rungameid/730"
    );
    for bad in ["0", "-1", "730?other=1", "730 & calc.exe", "4294967296"] {
        assert!(LaunchPlan::for_game(&game(StoreId::Steam, bad)).is_err());
    }
}

#[test]
fn epic_uses_all_three_identifiers_and_encodes_each_component() {
    let mut game = game(StoreId::Epic, "Fortnite");
    game.epic_identity = Some(EpicIdentity {
        namespace: "fn".into(),
        item_id: "fixture".into(),
    });
    assert_eq!(
        LaunchPlan::for_game(&game).unwrap().uri(),
        "com.epicgames.launcher://apps/fn%3Afixture%3AFortnite?action=launch&silent=true"
    );
    game.source_id = "name?x=1&y=2".into();
    assert!(
        LaunchPlan::for_game(&game)
            .unwrap()
            .uri()
            .contains("name%3Fx%3D1%26y%3D2?action=")
    );
    game.epic_identity = None;
    assert!(LaunchPlan::for_game(&game).is_err());
}

#[test]
fn unavailable_install_fails_before_any_os_handoff() {
    assert!(
        launch(&game(StoreId::Steam, "730"))
            .unwrap_err()
            .contains("folder is unavailable")
    );
}

#[test]
fn handoff_receives_the_store_uri_and_propagates_success_and_failure() {
    let temp = tempfile::TempDir::new().unwrap();
    let mut game = game(StoreId::Steam, "730");
    game.install_dir = temp.path().to_owned();
    assert!(
        launch_with(&game, |uri| {
            assert_eq!(uri, "steam://rungameid/730");
            Ok(())
        })
        .is_ok()
    );
    assert_eq!(
        launch_with(&game, |_| Err("No store handler".into())),
        Err("No store handler".into())
    );
}

#[test]
#[cfg(windows)]
fn windows_returns_a_missing_protocol_error_without_opening_a_dialog() {
    let error = platform::open_uri("planetes-fixture-unregistered-protocol://no-game").unwrap_err();
    assert!(error.contains("unavailable"), "{error}");
}
