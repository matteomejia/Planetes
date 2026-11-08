//! Typed store launch plans and operating-system handoff, independent of the UI.

mod platform;

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use planetes_core::{SourceGame, StoreId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    uri: String,
}

impl LaunchPlan {
    pub fn for_game(game: &SourceGame) -> Result<Self, String> {
        let uri = match game.store {
            StoreId::Steam => {
                let id: u32 = game
                    .source_id
                    .parse()
                    .map_err(|_| "Steam app ID is invalid.")?;
                if id == 0 {
                    return Err("Steam app ID is invalid.".into());
                }
                format!("steam://rungameid/{id}")
            }
            StoreId::Epic => {
                let identity = game
                    .epic_identity
                    .as_ref()
                    .ok_or("Epic launch identity is missing. Refresh your games and try again.")?;
                let fields = [&identity.namespace, &identity.item_id, &game.source_id];
                if fields
                    .iter()
                    .any(|s| s.trim().is_empty() || s.chars().any(char::is_control))
                {
                    return Err(
                        "Epic launch identity is incomplete. Refresh your games and try again."
                            .into(),
                    );
                }
                let encoded: Vec<_> = fields
                    .iter()
                    .map(|s| utf8_percent_encode(s, NON_ALPHANUMERIC).to_string())
                    .collect();
                format!(
                    "com.epicgames.launcher://apps/{}?action=launch&silent=true",
                    encoded.join("%3A")
                )
            }
        };
        Ok(Self { uri })
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
}

/// Success means the OS accepted the URI handoff, not that the game reached its first frame.
pub fn launch(game: &SourceGame) -> Result<(), String> {
    launch_with(game, platform::open_uri)
}

fn launch_with(
    game: &SourceGame,
    handoff: impl FnOnce(&str) -> Result<(), String>,
) -> Result<(), String> {
    let plan = LaunchPlan::for_game(game)?;
    if !game.install_dir.is_dir() {
        return Err("The game folder is unavailable. Refresh your games and try again.".into());
    }
    handoff(plan.uri())
}

#[cfg(test)]
mod tests;
