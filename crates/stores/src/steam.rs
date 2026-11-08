use std::path::{Component, Path};

use planetes_core::{ScanReport, SourceGame, StoreId};
use steamlocate::{Library, SteamDir};

pub(super) fn scan(root: Option<&Path>, report: &mut ScanReport) {
    let steam = match root.map_or_else(SteamDir::locate, SteamDir::from_dir) {
        Ok(steam) => steam,
        Err(error) => {
            report.warn(StoreId::Steam, root.map(Path::to_owned), error.to_string());
            return;
        }
    };
    let types = super::steam_types::read(steam.path(), report);
    let mut paths = match steam.library_paths() {
        Ok(paths) => paths,
        Err(error) => {
            report.warn(
                StoreId::Steam,
                Some(steam.path().join("steamapps/libraryfolders.vdf")),
                error.to_string(),
            );
            Vec::new()
        }
    };
    // The primary library remains usable if the library index is missing or corrupt.
    paths.push(steam.path().to_owned());
    paths.sort();
    paths.dedup();
    for path in paths {
        let library = match Library::from_dir(&path) {
            Ok(library) => library,
            Err(error) => {
                report.warn(StoreId::Steam, Some(path), error.to_string());
                continue;
            }
        };
        let mut ids = library.app_ids().to_vec();
        ids.sort();
        for id in ids {
            // Missing cache entries remain discoverable; only known non-games are excluded.
            if types
                .get(&id)
                .and_then(|metadata| metadata.kind.as_deref())
                .is_some_and(|kind| !matches!(kind, "game" | "demo"))
            {
                continue;
            }
            let manifest = library
                .path()
                .join("steamapps")
                .join(format!("appmanifest_{id}.acf"));
            let app = match library.app(id) {
                Some(Ok(app)) => app,
                Some(Err(error)) => {
                    report.warn(StoreId::Steam, Some(manifest), error.to_string());
                    continue;
                }
                None => continue,
            };
            if app.app_id != id
                || app.install_dir.is_empty()
                || Path::new(&app.install_dir)
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_)))
            {
                report.warn(
                    StoreId::Steam,
                    Some(manifest),
                    "Manifest app ID or relative install directory is invalid",
                );
                continue;
            }
            // FullyInstalled is bit 2. Other bits can also be set (e.g. an update is required).
            if !app.state_flags.is_some_and(|flags| flags.0 & 4 != 0) {
                continue;
            }
            let install_dir = library.resolve_app_dir(&app);
            // Soundtrack apps live under music, not common. They are not game installs.
            if !install_dir.is_dir()
                && library
                    .path()
                    .join("steamapps/music")
                    .join(&app.install_dir)
                    .is_dir()
            {
                continue;
            }
            if super::installed_directory(&install_dir, &manifest, StoreId::Steam, report) {
                report.games.push(SourceGame {
                    store: StoreId::Steam,
                    source_id: id.to_string(),
                    title: app
                        .name
                        .filter(|n| !n.trim().is_empty())
                        .unwrap_or_else(|| format!("Steam app {id}")),
                    install_dir,
                    manifest_path: manifest,
                    epic_identity: None,
                    icon_path: types
                        .get(&id)
                        .and_then(|metadata| metadata.icon_hash.as_ref())
                        .map(|hash| steam.path().join("steam/games").join(format!("{hash}.ico")))
                        .filter(|path| path.is_file()),
                });
            }
        }
    }
}
