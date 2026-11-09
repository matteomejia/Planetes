//! When each game was last launched through Planetes, in `%APPDATA%\Planetes\history.json`.
//! Epic keeps no local play history, and Steam writes its own only now and then, so a launch
//! from here shows up straight away. Only launch times are kept: no playtime tracking.
use planetes_core::{SourceGame, StoreId};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

type History = HashMap<String, u64>;

fn key(store: StoreId, source_id: &str) -> String {
    let store = match store {
        StoreId::Steam => "steam",
        StoreId::Epic => "epic",
    };
    format!("{store}:{source_id}")
}

fn path() -> Option<std::path::PathBuf> {
    crate::background::settings_path().map(|settings| settings.with_file_name("history.json"))
}

fn read() -> History {
    path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Record a launch. Failing to save only loses the detail, so errors are ignored.
pub fn record(game: &SourceGame) {
    let Some(path) = path() else {
        return;
    };
    let mut history = read();
    history.insert(key(game.store, &game.source_id), now());
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(&history) {
        let _ = std::fs::write(path, text);
    }
}

/// Use the later of the store's last played time and the last launch from Planetes.
pub fn merge(games: &mut [SourceGame]) {
    merge_from(games, &read());
}

pub(crate) fn merge_from(games: &mut [SourceGame], history: &History) {
    for game in games {
        if let Some(&launched) = history.get(&key(game.store, &game.source_id)) {
            game.last_played = game.last_played.max(Some(launched));
        }
    }
}
