#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

mod fixtures;
mod metrics;
mod platform;
mod state;
mod view;
mod visual;

use rune::{App, AppSpec, Key, KeyEvent, Tx};
use state::State;

fn on_key(s: &mut State, event: &KeyEvent, tx: &mut Tx) -> bool {
    match event.key {
        Key::Escape => tx.quit(),
        Key::Command('r') => s.refresh(),
        Key::Down => s.move_selection(1, tx),
        Key::Up => s.move_selection(-1, tx),
        Key::Enter => s.launch_selected(),
        _ => return false,
    }
    true
}

fn spec() -> AppSpec<State> {
    AppSpec {
        min_size: Some((420.0, 320.0)),
        on_key: Some(on_key),
        poll: Some(state::poll),
        setup: Some(state::setup),
        scenes: fixtures::SCENES,
        ..AppSpec::new("Planetes", (580.0, 480.0), State::default, view::view)
    }
}

fn main() {
    metrics::start();
    // Tooling and benchmark processes stay independent of the interactive application.
    if !metrics::enabled()
        && std::env::args().len() == 1
        && !rune::single_instance("io.github.matteomejia.planetes")
    {
        return;
    }
    let mut app = App::new(spec()).on_window(platform::set_icon);
    if metrics::enabled() {
        app = app.on_frame_drawn(metrics::frame);
    }
    let handle = app.handle();
    let ready = handle.clone();
    app.init(move || State {
        handle: Some(handle),
        ..State::default()
    })
    .on_ready(move |_| ready.update(|state, _| state.refresh()))
    .run();
}

#[cfg(test)]
mod tests;
