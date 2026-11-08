use planetes_core::{ScanReport, SourceGame};
use rune::{AppHandle, TextEdit, Tx};
use std::collections::HashMap;

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
        }
    }
}

impl State {
    pub fn apply_report(&mut self, report: ScanReport) {
        self.games = report.games;
        self.games
            .sort_by_key(|g| (g.title.to_lowercase(), g.store, g.source_id.clone()));
        self.warnings = report.warnings.len();
        self.loading = false;
        self.scanning = false;
        self.filter();
        self.load_icons();
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
            Ok(()) => tx.quit(),
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
        self.icons.clear();
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
            let image = cache.app_icon_scaled(path, 32.0, 2.0);
            self.icons
                .insert((game.store, game.source_id.clone()), image);
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
