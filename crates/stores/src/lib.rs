//! Read-only local discovery. No account sessions, network calls, or launcher writes.

mod epic;
mod epic_catalog;
mod steam;
mod steam_activity;
mod steam_types;

use std::{
    fs,
    path::{Path, PathBuf},
};

use planetes_core::{ScanReport, StoreId};

#[derive(Debug)]
pub struct ScanOptions {
    pub scan_steam: bool,
    pub scan_epic: bool,
    /// Overrides the Steam client root, not an individual library directory.
    pub steam_root: Option<PathBuf>,
    pub epic_manifests: Option<PathBuf>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            scan_steam: true,
            scan_epic: true,
            steam_root: None,
            epic_manifests: None,
        }
    }
}

pub fn scan(options: &ScanOptions) -> ScanReport {
    let mut report = ScanReport::default();
    if options.scan_steam {
        steam::scan(options.steam_root.as_deref(), &mut report);
    }
    if options.scan_epic {
        if let Some(path) = options.epic_manifests.as_deref() {
            epic::scan(path, true, &mut report);
        } else if let Some(path) = default_epic_manifests() {
            epic::scan(&path, false, &mut report);
        }
    }
    report.finish();
    report
}

fn default_epic_manifests() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("ProgramData")
            .map(|root| PathBuf::from(root).join("Epic/EpicGamesLauncher/Data/Manifests"))
    }
    // No unverified macOS path guess. Explicit manifest directories work on every platform.
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

fn installed_directory(
    path: &Path,
    manifest: &Path,
    store: StoreId,
    report: &mut ScanReport,
) -> bool {
    match fs::metadata(path) {
        Ok(meta) if meta.is_dir() => true,
        Ok(_) => {
            report.warn(
                store,
                Some(manifest.to_owned()),
                "Install location is not a directory",
            );
            false
        }
        Err(error) => {
            report.warn(
                store,
                Some(manifest.to_owned()),
                format!(
                    "Cannot access install directory {}: {error}",
                    path.display()
                ),
            );
            false
        }
    }
}
