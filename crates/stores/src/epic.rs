use std::{
    fs, io,
    path::{Path, PathBuf},
};

use planetes_core::{EpicIdentity, ScanReport, SourceGame, StoreId};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Manifest {
    display_name: String,
    app_name: String,
    #[serde(default)]
    catalog_namespace: String,
    #[serde(default)]
    catalog_item_id: String,
    install_location: String,
    #[serde(default)]
    launch_executable: String,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete: bool,
    #[serde(default, rename = "bIsApplication")]
    application: Option<bool>,
    #[serde(default, rename = "bIsExecutable")]
    executable: Option<bool>,
    #[serde(default)]
    app_categories: Vec<String>,
}

pub(super) fn scan(dir: &Path, explicit: bool, report: &mut ScanReport) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if !explicit && error.kind() == io::ErrorKind::NotFound => return,
        Err(error) => {
            report.warn(StoreId::Epic, Some(dir.to_owned()), error.to_string());
            return;
        }
    };
    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry)
                if entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("item")) =>
            {
                paths.push(entry.path())
            }
            Ok(_) => (),
            Err(error) => report.warn(StoreId::Epic, Some(dir.to_owned()), error.to_string()),
        }
    }
    paths.sort();
    for path in paths {
        let manifest = match fs::read(&path).map_err(|e| e.to_string()).and_then(|data| {
            // Tolerate launchers writing UTF-8 with a BOM.
            serde_json::from_slice::<Manifest>(data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&data))
                .map_err(|e| e.to_string())
        }) {
            Ok(manifest) => manifest,
            Err(error) => {
                report.warn(StoreId::Epic, Some(path), error);
                continue;
            }
        };
        if manifest.incomplete
            || manifest.application == Some(false)
            || manifest.executable == Some(false)
            || manifest.launch_executable.trim().is_empty()
            || (!manifest.app_categories.is_empty()
                && !manifest.app_categories.iter().any(|c| c == "games"))
        {
            continue;
        }
        if manifest.app_name.trim().is_empty()
            || manifest.display_name.trim().is_empty()
            || manifest.install_location.trim().is_empty()
        {
            report.warn(
                StoreId::Epic,
                Some(path),
                "Manifest has an empty app ID, title, or install location",
            );
            continue;
        }
        let install_dir = PathBuf::from(manifest.install_location);
        if !install_dir.is_absolute() {
            report.warn(
                StoreId::Epic,
                Some(path),
                "Install location must be an absolute path",
            );
            continue;
        }
        if super::installed_directory(&install_dir, &path, StoreId::Epic, report) {
            let relative = Path::new(&manifest.launch_executable);
            let icon_path = (!relative.as_os_str().is_empty()
                && relative
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))))
            .then(|| install_dir.join(relative))
            .filter(|path| path.is_file());
            report.games.push(SourceGame {
                store: StoreId::Epic,
                source_id: manifest.app_name,
                title: manifest.display_name,
                install_dir,
                icon_path,
                manifest_path: path,
                epic_identity: Some(EpicIdentity {
                    namespace: manifest.catalog_namespace,
                    item_id: manifest.catalog_item_id,
                }),
            });
        }
    }
}
