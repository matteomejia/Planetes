//! Store-independent records. Source IDs remain scoped to a store; cross-store merging comes later.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreId {
    Steam,
    Epic,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SourceGame {
    pub store: StoreId,
    pub source_id: String,
    pub title: String,
    pub install_dir: PathBuf,
    pub manifest_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epic_identity: Option<EpicIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_path: Option<PathBuf>,
    /// When the store last saw the game played, in Unix seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<u64>,
    /// Total playtime the store recorded, in minutes. Planetes doesn't track playtime itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playtime_minutes: Option<u64>,
    /// Wide key art the store keeps locally (Steam's library hero), about 3:1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_path: Option<PathBuf>,
    /// The game's logo with a transparent background, drawn over the hero art.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_path: Option<PathBuf>,
    /// Where the store's own CDN serves key art the store doesn't keep locally (Epic). The
    /// app downloads it once into its art cache, then uses it as `hero_path`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EpicIdentity {
    pub namespace: String,
    pub item_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScanWarning {
    pub store: StoreId,
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ScanReport {
    pub games: Vec<SourceGame>,
    pub warnings: Vec<ScanWarning>,
}

impl ScanReport {
    pub fn warn(&mut self, store: StoreId, path: Option<PathBuf>, message: impl Into<String>) {
        self.warnings.push(ScanWarning {
            store,
            path,
            message: message.into(),
        });
    }

    /// Prefer the first manifest path deterministically when stale duplicate records exist.
    pub fn finish(&mut self) {
        self.games.sort_by(|a, b| {
            (a.store, &a.source_id, &a.manifest_path).cmp(&(
                b.store,
                &b.source_id,
                &b.manifest_path,
            ))
        });
        self.games
            .dedup_by(|a, b| a.store == b.store && a.source_id == b.source_id);
        self.warnings
            .sort_by(|a, b| (a.store, &a.path, &a.message).cmp(&(b.store, &b.path, &b.message)));
    }
}
