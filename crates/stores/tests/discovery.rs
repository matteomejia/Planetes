use std::{fs, path::Path};

use planetes_core::StoreId;
use planetes_stores::{ScanOptions, scan};
use serde_json::{Value, json};
use tempfile::TempDir;

#[path = "fixtures/appinfo.rs"]
mod appinfo;

const STEAM_APP: &str = include_str!("fixtures/steam-installed.acf");

#[test]
fn steam_icon_discovery_uses_only_existing_cached_icons_with_valid_hashes() {
    for version in [0x28, 0x29] {
        let temp = TempDir::new().unwrap();
        steam_app(temp.path(), 100, 4, true);
        libraries(temp.path(), &[temp.path()]);
        fs::create_dir_all(temp.path().join("appcache")).unwrap();
        fs::create_dir_all(temp.path().join("steam/games")).unwrap();
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let path = temp.path().join(format!("steam/games/{hash}.ico"));
        fs::write(&path, b"fixture; discovery does not decode").unwrap();
        fs::write(
            temp.path().join("appcache/appinfo.vdf"),
            appinfo::appinfo_with_icon(version, &[(100, "Game")], Some(hash)),
        )
        .unwrap();
        assert_eq!(
            scan(&steam_options(temp.path())).games[0]
                .icon_path
                .as_ref(),
            Some(&path)
        );
        fs::remove_file(&path).unwrap();
        assert!(
            scan(&steam_options(temp.path())).games[0]
                .icon_path
                .is_none()
        );
        fs::write(
            temp.path().join("appcache/appinfo.vdf"),
            appinfo::appinfo_with_icon(version, &[(100, "Game")], Some("../unexpected")),
        )
        .unwrap();
        assert!(
            scan(&steam_options(temp.path())).games[0]
                .icon_path
                .is_none()
        );
    }
}

#[test]
fn steam_classifies_games_demos_and_non_games_in_both_cache_versions() {
    for version in [0x28, 0x29] {
        let temp = TempDir::new().unwrap();
        for id in 100..105 {
            steam_app(temp.path(), id, 4, true);
        }
        libraries(temp.path(), &[temp.path()]);
        fs::create_dir(temp.path().join("appcache")).unwrap();
        fs::write(
            temp.path().join("appcache/appinfo.vdf"),
            appinfo::appinfo(
                version,
                &[
                    (100, "Game"),
                    (101, "Demo"),
                    (102, "Tool"),
                    (103, "Application"),
                ],
            ),
        )
        .unwrap();
        let report = scan(&steam_options(temp.path()));
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(
            report
                .games
                .iter()
                .map(|g| g.source_id.as_str())
                .collect::<Vec<_>>(),
            ["100", "101", "104"]
        );
    }
}

#[test]
fn steam_play_history_comes_from_the_account_that_played_last() {
    let temp = TempDir::new().unwrap();
    for id in [100, 101, 102] {
        steam_app(temp.path(), id, 4, true);
    }
    libraries(temp.path(), &[temp.path()]);
    let config = |account: &str, apps: &str| {
        let dir = temp.path().join("userdata").join(account).join("config");
        fs::create_dir_all(&dir).unwrap();
        let text = format!(
            "\"UserLocalConfigStore\"\n{{\n\t\"Software\"\n\t{{\n\t\t\"valve\"\n\t\t{{\n\t\t\t\"Steam\"\n\t\t\t{{\n\t\t\t\t\"Apps\"\n\t\t\t\t{{\n{apps}\t\t\t\t}}\n\t\t\t}}\n\t\t}}\n\t}}\n}}\n"
        );
        fs::write(dir.join("localconfig.vdf"), text).unwrap();
    };
    config(
        "1",
        "\"100\" { \"LastPlayed\" \"1700000000\" \"Playtime\" \"90\" }\n\"101\" { \"LastPlayed\" \"0\" \"Playtime\" \"0\" }\n",
    );
    config(
        "2",
        "\"100\" { \"LastPlayed\" \"1800000000\" \"Playtime\" \"5\" }\n",
    );
    fs::create_dir_all(temp.path().join("userdata/3/config")).unwrap();
    fs::write(
        temp.path().join("userdata/3/config/localconfig.vdf"),
        "\"broken",
    )
    .unwrap();
    let report = scan(&steam_options(temp.path()));
    let activity: Vec<_> = report
        .games
        .iter()
        .map(|g| (g.source_id.as_str(), g.last_played, g.playtime_minutes))
        .collect();
    assert_eq!(
        activity,
        [
            ("100", Some(1_800_000_000), Some(5)),
            ("101", None, None),
            ("102", None, None)
        ]
    );
    assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
}

#[test]
fn broken_steam_cache_warns_but_preserves_manifest_discovery() {
    let temp = TempDir::new().unwrap();
    steam_app(temp.path(), 100, 4, true);
    libraries(temp.path(), &[temp.path()]);
    fs::create_dir(temp.path().join("appcache")).unwrap();
    fs::write(temp.path().join("appcache/appinfo.vdf"), b"broken").unwrap();
    let report = scan(&steam_options(temp.path()));
    assert_eq!(report.games.len(), 1);
    assert_eq!(report.warnings.len(), 1);
}

fn steam_options(root: &Path) -> ScanOptions {
    ScanOptions {
        scan_epic: false,
        steam_root: Some(root.to_owned()),
        ..Default::default()
    }
}

fn steam_app(root: &Path, id: u32, flags: u32, installed: bool) {
    let common = root.join("steamapps/common");
    fs::create_dir_all(&common).unwrap();
    let content = STEAM_APP
        .replace("\"100\"", &format!("\"{id}\""))
        .replace("\"4\"", &format!("\"{flags}\""));
    fs::write(
        root.join(format!("steamapps/appmanifest_{id}.acf")),
        content,
    )
    .unwrap();
    if installed {
        fs::create_dir_all(common.join("FixtureGame")).unwrap();
    }
}

fn libraries(root: &Path, paths: &[&Path]) {
    fs::create_dir_all(root.join("steamapps")).unwrap();
    let rows: String = paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let path = path
                .to_string_lossy()
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            format!("\"{i}\" {{ \"path\" \"{path}\" \"apps\" {{ }} }}\n")
        })
        .collect();
    fs::write(
        root.join("steamapps/libraryfolders.vdf"),
        format!("\"libraryfolders\" {{ {rows} }}"),
    )
    .unwrap();
}

#[test]
fn steam_reads_multiple_libraries_and_update_required_install() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("Steam");
    let second = temp.path().join("Second library");
    steam_app(&root, 100, 4, true);
    steam_app(&second, 200, 6, true); // installed + update required
    libraries(&root, &[&root, &second]);
    let report = scan(&steam_options(&root));
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(
        report
            .games
            .iter()
            .map(|g| g.source_id.as_str())
            .collect::<Vec<_>>(),
        ["100", "200"]
    );
    assert_eq!(
        report.games[1].install_dir,
        second.join("steamapps/common/FixtureGame")
    );
}

#[test]
fn steam_continues_after_bad_manifest_and_missing_library() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    steam_app(root, 100, 4, true);
    steam_app(root, 200, 1024, true); // download only
    fs::write(root.join("steamapps/appmanifest_300.acf"), "broken").unwrap();
    libraries(root, &[root, &root.join("offline")]);
    let report = scan(&steam_options(root));
    assert_eq!(report.games.len(), 1);
    assert_eq!(report.warnings.len(), 2);
}

#[test]
fn steam_recovers_primary_library_from_broken_index_and_rejects_stale_install() {
    let temp = TempDir::new().unwrap();
    steam_app(temp.path(), 100, 4, true);
    fs::write(temp.path().join("steamapps/libraryfolders.vdf"), "broken").unwrap();
    let report = scan(&steam_options(temp.path()));
    assert_eq!(report.games.len(), 1);
    assert_eq!(report.warnings.len(), 1);
    fs::remove_dir(temp.path().join("steamapps/common/FixtureGame")).unwrap();
    let report = scan(&steam_options(temp.path()));
    assert!(report.games.is_empty());
    assert_eq!(report.warnings.len(), 2);
}

#[test]
fn steam_rejects_mismatched_ids_and_paths_outside_common() {
    let temp = TempDir::new().unwrap();
    steam_app(temp.path(), 100, 4, true);
    libraries(temp.path(), &[temp.path()]);
    fs::write(temp.path().join("steamapps/appmanifest_200.acf"), STEAM_APP).unwrap();
    fs::write(
        temp.path().join("steamapps/appmanifest_100.acf"),
        STEAM_APP.replace("FixtureGame", "../escape"),
    )
    .unwrap();
    let report = scan(&steam_options(temp.path()));
    assert!(report.games.is_empty());
    assert_eq!(report.warnings.len(), 2);
}

#[test]
fn steam_skips_installed_music_without_reporting_a_stale_game() {
    let temp = TempDir::new().unwrap();
    steam_app(temp.path(), 100, 4, false);
    libraries(temp.path(), &[temp.path()]);
    fs::create_dir_all(temp.path().join("steamapps/music/FixtureGame")).unwrap();
    let report = scan(&steam_options(temp.path()));
    assert!(report.games.is_empty());
    assert!(report.warnings.is_empty());
}

fn epic_manifest(dir: &Path, file: &str, overrides: Value) {
    let mut manifest: Value =
        serde_json::from_str(include_str!("fixtures/epic-game.item")).unwrap();
    manifest["InstallLocation"] = json!(dir.join("game"));
    for (key, value) in overrides.as_object().unwrap() {
        manifest[key] = value.clone();
    }
    fs::write(dir.join(file), serde_json::to_vec(&manifest).unwrap()).unwrap();
}

fn epic_options(path: &Path) -> ScanOptions {
    ScanOptions {
        scan_steam: false,
        epic_manifests: Some(path.to_owned()),
        ..Default::default()
    }
}

#[test]
fn epic_filters_content_incomplete_and_non_game_manifests_and_deduplicates() {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("game")).unwrap();
    epic_manifest(
        temp.path(),
        "game.item",
        json!({"DisplayName": "Fixturé Game"}),
    );
    epic_manifest(
        temp.path(),
        "duplicate.item",
        json!({"DisplayName": "Fixturé Game"}),
    );
    epic_manifest(
        temp.path(),
        "content.item",
        json!({"AppName": "content", "bIsApplication": false, "LaunchExecutable": "", "AppCategories": ["hidden"]}),
    );
    epic_manifest(
        temp.path(),
        "download.item",
        json!({"AppName": "download", "bIsIncompleteInstall": true}),
    );
    epic_manifest(
        temp.path(),
        "engine.item",
        json!({"AppName": "engine", "AppCategories": ["engines"]}),
    );
    epic_manifest(
        temp.path(),
        "not-executable.item",
        json!({"AppName": "content2", "bIsExecutable": false}),
    );
    let report = scan(&epic_options(temp.path()));
    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert_eq!(report.games.len(), 1);
    assert_eq!(report.games[0].store, StoreId::Epic);
    assert_eq!(report.games[0].title, "Fixturé Game");
}

#[test]
fn epic_retains_valid_entries_when_manifests_are_bad_or_stale() {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("game")).unwrap();
    epic_manifest(temp.path(), "good.item", json!({}));
    epic_manifest(
        temp.path(),
        "stale.item",
        json!({"AppName": "stale", "InstallLocation": temp.path().join("missing")}),
    );
    epic_manifest(temp.path(), "empty.item", json!({"AppName": ""}));
    epic_manifest(
        temp.path(),
        "relative.item",
        json!({"AppName": "relative", "InstallLocation": "."}),
    );
    fs::write(temp.path().join("bad.item"), b"{invalid").unwrap();
    fs::write(temp.path().join("ignored.json"), b"invalid").unwrap();
    let report = scan(&epic_options(temp.path()));
    assert_eq!(report.games.len(), 1);
    assert_eq!(report.warnings.len(), 4);
}

#[test]
fn epic_accepts_bom_and_optional_legacy_flags() {
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join("game")).unwrap();
    epic_manifest(temp.path(), "legacy.ITEM", json!({}));
    let path = temp.path().join("legacy.ITEM");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for field in ["bIsApplication", "bIsExecutable", "AppCategories"] {
        manifest.as_object_mut().unwrap().remove(field);
    }
    let mut data = b"\xef\xbb\xbf".to_vec();
    data.extend(serde_json::to_vec(&manifest).unwrap());
    fs::write(path, data).unwrap();
    let report = scan(&epic_options(temp.path()));
    assert_eq!(report.games.len(), 1);
    assert!(report.warnings.is_empty());
}

#[test]
fn empty_directory_is_empty_but_explicit_missing_path_warns() {
    let temp = TempDir::new().unwrap();
    let empty = scan(&epic_options(temp.path()));
    assert!(empty.games.is_empty() && empty.warnings.is_empty());
    let missing = scan(&epic_options(&temp.path().join("missing")));
    assert!(missing.games.is_empty());
    assert_eq!(missing.warnings.len(), 1);
}
