//! Opt-in native release measurements; no logging or timers in ordinary app sessions.
use crate::state::State;
use rune::{Runtime, Tx};
use std::{
    cell::Cell,
    path::PathBuf,
    sync::OnceLock,
    time::{Duration, Instant},
};

static START: OnceLock<Instant> = OnceLock::new();
pub fn start() {
    let _ = START.set(Instant::now());
}
pub fn enabled() -> bool {
    std::env::var_os("PLANETES_BENCHMARK_OUTPUT").is_some()
}

pub(crate) struct Benchmark {
    output: PathBuf,
    started: Instant,
    first_frame: Cell<Option<f64>>,
    library_frame: Cell<Option<f64>>,
    library_at: Cell<Option<Instant>>,
    timer_set: bool,
}

impl Benchmark {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            output: std::env::var_os("PLANETES_BENCHMARK_OUTPUT")?.into(),
            started: *START.get_or_init(Instant::now),
            first_frame: Cell::new(None),
            library_frame: Cell::new(None),
            library_at: Cell::new(None),
            timer_set: false,
        })
    }
}

pub(crate) fn frame(rt: &Runtime<State>) {
    let Some(bench) = &rt.state.benchmark else {
        return;
    };
    let elapsed = bench.started.elapsed().as_secs_f64() * 1000.0;
    if bench.first_frame.get().is_none() {
        bench.first_frame.set(Some(elapsed));
    }
    if !rt.state.loading && bench.library_frame.get().is_none() {
        bench.library_frame.set(Some(elapsed));
        bench.library_at.set(Some(Instant::now()));
        if let Some(handle) = &rt.state.handle {
            handle.wake();
        }
    }
}

pub(crate) fn poll(state: &mut State, tx: &mut Tx) {
    let Some(bench) = &mut state.benchmark else {
        return;
    };
    let Some(library_at) = bench.library_at.get() else {
        return;
    };
    if !bench.timer_set {
        bench.timer_set = true;
        tx.wake_after(Duration::from_secs(2));
        return;
    }
    if library_at.elapsed() < Duration::from_secs(2) {
        return;
    }
    let bench = state.benchmark.take().expect("benchmark was present");
    let saved_query = state.query.text.clone();
    let saved_selected = state.selected;
    let mut search_us = Vec::with_capacity(1000);
    for query in ["a", "er", "zzzz", "2077", "fort"] {
        state.query.text = query.into();
        for _ in 0..200 {
            let before = Instant::now();
            state.filter();
            search_us.push(before.elapsed().as_secs_f64() * 1_000_000.0);
        }
    }
    state.query.text = saved_query;
    state.filter();
    state.selected = saved_selected;
    search_us.sort_by(f64::total_cmp);
    let result = serde_json::json!({
        "first_frame_cpu_ms": bench.first_frame.get(), "library_frame_cpu_ms": bench.library_frame.get(),
        "search_cpu_us_p50": search_us[search_us.len()/2],
        "search_cpu_us_p95": search_us[search_us.len()*95/100],
        "search_samples": search_us.len(), "games": state.games.len(), "warnings": state.warnings,
        "icons_ready": state.icons_ready,
        "method": "Native frame callback from main entry; CPU submission, not scanout. Search measures filtering only. Fresh process; OS/driver caches retained."
    });
    if let Err(error) = std::fs::write(&bench.output, result.to_string()) {
        eprintln!("Could not write benchmark output: {error}");
    }
    tx.quit();
}
