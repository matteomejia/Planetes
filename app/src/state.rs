use planetes_core::{ScanReport, SourceGame};
use rune::{AppHandle, TextEdit, Tx};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// A summon rescans when the library is older than this, so newly installed games appear.
const RESCAN_AFTER: Duration = Duration::from_secs(60);

pub(crate) struct State {
    pub query: TextEdit,
    pub games: Vec<SourceGame>,
    pub results: Vec<usize>,
    pub selected: usize,
    pub loading: bool,
    pub launching: bool,
    pub error: Option<String>,
    pub warnings: usize,
    pub handle: Option<AppHandle<State>>,
    pub pending_launch: Option<SourceGame>,
    pub scanning: bool,
    pub image_cache: Option<rune::ImageCache>,
    pub icons: HashMap<(planetes_core::StoreId, String), rune::Image>,
    pub icons_ready: usize,
    pub benchmark: Option<crate::metrics::Benchmark>,
    /// Background mode: dismissing hides the window instead of quitting.
    pub resident: bool,
    pub hotkey: Option<(rune::ShortcutId, String)>,
    /// A settings or hotkey problem, shown quietly above the status line.
    pub notice: Option<String>,
    pub tray: Option<rune::Tray>,
    pub last_scan: Option<Instant>,
}

impl Default for State {
    fn default() -> Self {
        let mut query = TextEdit::new("");
        query.focused = true;
        Self {
            query,
            games: Vec::new(),
            results: Vec::new(),
            selected: 0,
            loading: true,
            launching: false,
            error: None,
            warnings: 0,
            handle: None,
            pending_launch: None,
            scanning: false,
            image_cache: None,
            icons: HashMap::new(),
            icons_ready: 0,
            benchmark: crate::metrics::Benchmark::from_env(),
            resident: false,
            hotkey: None,
            notice: None,
            tray: None,
            last_scan: None,
        }
    }
}

impl State {
    pub fn apply_report(&mut self, report: ScanReport) {
        let selected = self.selected_game().map(|g| (g.store, g.source_id.clone()));
        self.games = report.games;
        self.games
            .sort_by_key(|g| (g.title.to_lowercase(), g.store, g.source_id.clone()));
        self.warnings = report.warnings.len();
        self.loading = false;
        self.scanning = false;
        self.last_scan = Some(Instant::now());
        self.filter();
        // A rescan while the window is open must not move the selection under the user.
        if let Some(key) = selected
            && let Some(position) = self
                .results
                .iter()
                .position(|&i| (self.games[i].store, &self.games[i].source_id) == (key.0, &key.1))
        {
            self.selected = position;
        }
        self.load_icons();
    }

    fn selected_game(&self) -> Option<&SourceGame> {
        self.results.get(self.selected).map(|&i| &self.games[i])
    }

    /// Start background services once the app host runs: hotkey, settings watcher, first scan.
    pub fn start(&mut self) {
        if self.resident {
            self.apply_settings();
            if let Some(handle) = self.handle.clone() {
                crate::background::watch_settings(handle);
            }
        }
        self.refresh();
    }

    /// Show the launcher on the monitor under the pointer, with a fresh search.
    pub fn summon(&mut self) {
        let Some(handle) = self.handle.clone() else {
            return;
        };
        crate::platform::place_for_summon();
        handle.open();
        if self.last_scan.is_none_or(|at| at.elapsed() >= RESCAN_AFTER) {
            self.refresh();
        }
    }

    /// Leave Planetes: hide it in background mode, otherwise quit.
    pub fn dismiss(&mut self, tx: &mut Tx) {
        if !self.resident {
            tx.quit();
            return;
        }
        // Reset while hidden so the next summon never flashes the previous search.
        self.query = TextEdit::new("");
        self.query.focused = true;
        self.error = None;
        self.filter();
        if let Some(&index) = self.results.first() {
            tx.scroll_into_view(format!("game-{index}"));
        }
        if let Some(handle) = &self.handle {
            handle.hide();
        }
    }

    /// The hotkey: summon, or dismiss when already showing.
    pub fn toggle(&mut self, tx: &mut Tx) {
        if self.handle.as_ref().is_some_and(|h| h.is_visible()) {
            self.dismiss(tx);
        } else {
            self.summon();
        }
    }

    pub fn filter(&mut self) {
        let query = normalized(&self.query.text);
        let mut matches: Vec<_> = self
            .games
            .iter()
            .enumerate()
            .filter_map(|(index, game)| {
                let title = normalized(&game.title);
                let score = if query.is_empty() || title.starts_with(&query) {
                    0
                } else if title.contains(&query) {
                    1
                } else if is_subsequence(&query, &title) {
                    2
                } else {
                    return None;
                };
                Some((score, index))
            })
            .collect();
        matches.sort();
        self.results = matches.into_iter().map(|(_, index)| index).collect();
        self.selected = 0;
    }

    pub fn move_selection(&mut self, direction: isize, tx: &mut Tx) {
        if self.results.is_empty() {
            return;
        }
        self.selected =
            (self.selected as isize + direction).rem_euclid(self.results.len() as isize) as usize;
        tx.scroll_into_view(format!("game-{}", self.results[self.selected]));
    }

    pub fn launch_selected(&mut self) {
        if self.launching {
            return;
        }
        let Some(&index) = self.results.get(self.selected) else {
            return;
        };
        self.launch_game(self.games[index].clone());
    }

    pub fn launch_game(&mut self, game: SourceGame) {
        if self.launching {
            return;
        }
        match planetes_launcher::LaunchPlan::for_game(&game) {
            Ok(_) => {
                self.error = None;
                self.launching = true;
                self.pending_launch = Some(game);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn finish_launch(&mut self, result: Result<(), String>, tx: &mut Tx) {
        self.launching = false;
        match result {
            Ok(()) => self.dismiss(tx),
            Err(error) => self.error = Some(error),
        }
    }

    pub fn refresh(&mut self) {
        if self.launching || self.scanning {
            return;
        }
        let Some(handle) = self.handle.clone() else {
            return;
        };
        self.loading = true;
        self.scanning = true;
        self.error = None;
        std::thread::spawn(move || {
            let report = planetes_stores::scan(&Default::default());
            handle.update(move |state, tx| {
                state.apply_report(report);
                tx.instant();
            });
        });
    }

    fn load_icons(&mut self) {
        // At most 128 retained 64px icon rasters: about 2 MiB plus mip levels. No cover art.
        // Icons of games still installed are kept, so a rescan doesn't flash letter tiles.
        let mut previous = std::mem::take(&mut self.icons);
        self.icons_ready = 0;
        let Some(cache) = &self.image_cache else {
            return;
        };
        for game in self
            .games
            .iter()
            .filter(|g| g.icon_path.is_some())
            .take(128)
        {
            let path = game.icon_path.as_ref().expect("filtered icon path");
            let key = (game.store, game.source_id.clone());
            let image = previous
                .remove(&key)
                .unwrap_or_else(|| cache.app_icon_scaled(path, 32.0, 2.0));
            self.icons.insert(key, image);
        }
    }
}

pub(crate) fn setup(state: &mut State, waker: rune::Waker) {
    state.image_cache = Some(rune::ImageCache::new(waker));
    state.load_icons();
}

pub(crate) fn poll(state: &mut State, _: f64, tx: &mut Tx) -> bool {
    crate::metrics::poll(state, tx);
    // Fallback tiles don't bind pending pixels. Rebuild once when decoded icons can replace them.
    let ready = state
        .icons
        .values()
        .filter(|image| image.is_ready())
        .count();
    let changed = ready != state.icons_ready;
    state.icons_ready = ready;
    let Some(handle) = state.handle.clone() else {
        return changed;
    };
    let Some(game) = state.pending_launch.take() else {
        return changed;
    };
    // Tooling scripts can inspect states and input without opening external applications.
    let headless = rune::tooling::headless_run().is_some();
    std::thread::spawn(move || {
        let result = if headless {
            Err("Game launching is disabled in headless previews.".into())
        } else {
            planetes_launcher::launch(&game)
        };
        handle.update(move |state, tx| state.finish_launch(result, tx));
    });
    changed
}

fn normalized(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn is_subsequence(query: &str, title: &str) -> bool {
    let mut letters = title.chars();
    query
        .chars()
        .all(|wanted| letters.by_ref().any(|letter| letter == wanted))
}

/// Background mode: clicking away hides the launcher, as a command palette does.
pub(crate) fn on_focus(state: &mut State, focused: bool, tx: &mut Tx) {
    if !focused && state.handle.as_ref().is_some_and(|h| h.is_visible()) {
        state.dismiss(tx);
    }
}

/// Launching Planetes again while it runs summons the resident window.
pub(crate) fn on_reopen(state: &mut State, _: bool, _: &mut Tx) -> bool {
    state.summon();
    true
}
