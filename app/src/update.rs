//! Self-updates for copies installed by Setup.exe (or Velopack's portable ZIP) from GitHub
//! Releases. A copy run from `cargo run` or an unpacked v0.1 ZIP isn't installed and never
//! checks. An update downloads in the background and applies while the launcher is hidden,
//! restarting Planetes in background mode.
use crate::{background, state::State};
use rune::{AppHandle, Tx};
use std::time::Duration;
use velopack::{UpdateCheck, UpdateManager, VelopackApp, VelopackAsset, sources::AutoSource};

const RELEASES: &str = "https://github.com/matteomejia/Planetes";
/// Leaves start-up, and sign-in when Planetes starts with Windows, to everything else.
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// Velopack's install, update and uninstall hooks. Runs first in `main`: during those steps it
/// may handle the call and exit, and at start-up it applies an update already downloaded.
pub fn startup() {
    let app = VelopackApp::build();
    // The Run value would otherwise point at the removed executable. (A Windows-only hook.)
    #[cfg(windows)]
    let app = app.on_before_uninstall_fast_callback(|_| {
        let _ = crate::platform::set_autostart(false);
    });
    app.run();
}

/// An update downloaded and ready to apply.
pub(crate) struct Pending {
    manager: UpdateManager,
    asset: VelopackAsset,
}

/// Check now and then every day, downloading the first newer release found.
pub fn watch(handle: AppHandle<State>) {
    // `PLANETES_UPDATE_SOURCE` points at a local release folder or another URL, for testing.
    let source = std::env::var("PLANETES_UPDATE_SOURCE").unwrap_or_else(|_| RELEASES.into());
    let Ok(manager) = UpdateManager::new(AutoSource::new(&source), None, None) else {
        return; // Not an installed copy.
    };
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        loop {
            if enabled()
                && let Some(asset) = download(&manager)
            {
                let manager = manager.clone();
                handle.update(move |state, tx| state.update_ready(Pending { manager, asset }, tx));
                return;
            }
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

fn download(manager: &UpdateManager) -> Option<VelopackAsset> {
    let UpdateCheck::UpdateAvailable(update) = manager.check_for_updates().ok()? else {
        return None;
    };
    manager.download_updates(&update, None).ok()?;
    Some(update.TargetFullRelease)
}

/// `"check_for_updates": false` in settings.json turns checks off. Read at each check, so a
/// change applies without a restart; a missing or unreadable file keeps the default (on).
fn enabled() -> bool {
    background::settings_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .is_none_or(|text| updates_from_json(&text))
}

pub(crate) fn updates_from_json(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| value.get("check_for_updates")?.as_bool())
        .unwrap_or(true)
}

impl State {
    fn update_ready(&mut self, pending: Pending, tx: &mut Tx) {
        self.pending_update = Some(pending);
        if !self.handle.as_ref().is_some_and(|h| h.is_visible()) {
            self.apply_update(tx);
        }
    }

    /// Hand over to Velopack's updater, which waits for this process to exit, replaces the
    /// app and starts it again hidden. Called only while the launcher is hidden.
    pub(crate) fn apply_update(&mut self, tx: &mut Tx) {
        let Some(Pending { manager, asset }) = self.pending_update.take() else {
            return;
        };
        match manager.wait_exit_then_apply_updates(&asset, true, true, ["--background"]) {
            Ok(()) => tx.quit(),
            Err(error) => self.notice = Some(format!("Could not install the update: {error}.")),
        }
    }
}
