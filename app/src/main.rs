#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

mod art;
mod background;
mod fixtures;
mod history;
mod metrics;
mod platform;
mod preview;
mod state;
mod update;
mod view;
mod visual;

use rune::{App, AppSpec, Key, KeyEvent, Tx, WindowOptions};
use state::State;

fn on_key(s: &mut State, event: &KeyEvent, tx: &mut Tx) -> bool {
    match event.key {
        Key::Escape => s.dismiss(tx),
        Key::Command('r') => s.refresh(),
        Key::Down => s.move_selection(1, tx),
        Key::Up => s.move_selection(-1, tx),
        Key::Enter => s.launch_selected(),
        _ => return false,
    }
    true
}

/// Wide enough for the results list and the selected game's art beside it.
pub(crate) const SIZE: (f32, f32) = (860.0, 520.0);

fn spec() -> AppSpec<State> {
    AppSpec {
        min_size: Some((720.0, 400.0)),
        on_key: Some(on_key),
        poll: Some(state::poll),
        setup: Some(state::setup),
        scenes: fixtures::SCENES,
        ..AppSpec::new("Planetes", (SIZE.0, SIZE.1), State::default, view::view)
    }
}

fn main() {
    update::startup();
    metrics::start();
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Started hidden, as Windows does at sign-in when "Start with Windows" is on.
    let background = args.iter().any(|arg| arg == "--background");
    // Tooling (scenes, snapshots, scripts) and benchmark processes stay independent of the
    // interactive application: no single-instance claim, tray or hotkey, and Escape quits.
    let tooling = metrics::enabled() || args.iter().any(|arg| arg != "--background");
    if !tooling && !rune::single_instance("io.github.matteomejia.planetes") {
        // The running instance gets `on_reopen` and shows its window.
        return;
    }
    // Frameless like a command palette: no title bar or caption buttons. Esc dismisses it and
    // empty parts of the search and status bands move it.
    let mut app = App::new(spec())
        .window_options(WindowOptions {
            borderless: true,
            resizable: false,
            // A background start still creates the window, hidden, so the first summon shows
            // an existing window (which Windows lets take focus) without GPU start-up.
            visible: !background,
            ..WindowOptions::default()
        })
        .on_window(platform::setup_window);
    if metrics::enabled() {
        app = app.on_frame_drawn(metrics::frame);
    }
    let resident = !tooling;
    if resident {
        app = app
            .keep_running()
            .on_focus(state::on_focus)
            .on_reopen(state::on_reopen);
    }
    let handle = app.handle();
    let tray = resident.then(|| background::tray(handle.clone())).flatten();
    // Runs once the host exists, with or without a window, so the hotkey can register.
    handle.update(|state, _| state.start());
    app.init(move || State {
        handle: Some(handle),
        resident,
        tray,
        ..State::default()
    })
    .run();
}

#[cfg(test)]
mod tests;
