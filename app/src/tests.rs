use super::*;
use rune::{App, Env, KeyEvent, Mods, Runtime};

fn runtime() -> Runtime<State> {
    let mut rt = App::new(spec()).headless_runtime((580.0, 480.0), Env::new(true, 1.0));
    fixtures::library(&mut rt.state);
    rt.frame(0.0);
    rt
}
fn key(rt: &mut Runtime<State>, key: Key) {
    rt.key(KeyEvent {
        key,
        mods: Mods::default(),
    });
    rt.frame(0.0);
}

#[test]
fn typing_filters_and_enter_requests_the_selected_store_game_once() {
    let mut rt = runtime();
    key(&mut rt, Key::Text("fort".into()));
    assert_eq!(rt.state.query.text, "fort");
    assert_eq!(rt.state.results.len(), 1);
    key(&mut rt, Key::Enter);
    assert_eq!(
        rt.state.pending_launch.as_ref().unwrap().source_id,
        "Fortnite"
    );
    assert!(rt.state.launching);
    key(&mut rt, Key::Enter);
    assert_eq!(
        rt.state.pending_launch.as_ref().unwrap().source_id,
        "Fortnite"
    );
}

#[test]
fn arrows_wrap_and_keep_the_selected_row_in_view() {
    let mut rt = runtime();
    key(&mut rt, Key::Up);
    assert_eq!(rt.state.selected, rt.state.results.len() - 1);
    let selected = rt.state.results[rt.state.selected];
    let (_, y) = rt.find(&format!("game-{selected}")).unwrap();
    assert!(
        (100.0..440.0).contains(&y),
        "selected row must be visible: {y}"
    );
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.selected, 0);
}

#[test]
fn reduced_motion_applies_selection_without_an_animated_fill() {
    let mut rt = runtime();
    rt.set_system_reduced_motion(true);
    rt.frame(0.0);
    key(&mut rt, Key::Down);
    let wanted = format!("game-{}", rt.state.results[rt.state.selected]);
    let element = rt
        .elements()
        .into_iter()
        .find(|element| element.key.as_deref() == Some(&wanted))
        .unwrap();
    assert_eq!(element.fill, visual::palette().selected);
    assert!(!element.moving);
}

#[test]
fn click_requests_the_clicked_game_and_launch_failure_remains_visible() {
    let mut rt = runtime();
    let index = rt
        .state
        .games
        .iter()
        .position(|g| g.title == "Balatro")
        .unwrap();
    let (x, y) = rt.find(&format!("game-{index}")).unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(0.0);
    assert_eq!(rt.state.pending_launch.take().unwrap().title, "Balatro");
    rt.update(|state, tx| state.finish_launch(Err("Store is unavailable".into()), tx));
    rt.frame(0.0);
    assert!(rt.find("launch-error").is_some());
    assert!(!rt.state.launching);
}

#[test]
fn search_matches_abbreviations_and_punctuation_without_empty_selection_panics() {
    let mut state = State::default();
    fixtures::library(&mut state);
    state.query.text = "bg3".into();
    state.filter();
    assert_eq!(state.games[state.results[0]].title, "Baldur's Gate 3");
    state.query.text = "zzzzzz".into();
    state.filter();
    state.launch_selected();
    state.move_selection(1, &mut Tx::default());
    assert!(state.pending_launch.is_none());
}

#[test]
fn escape_and_successful_handoff_request_quit_but_errors_do_not() {
    let mut rt = runtime();
    key(&mut rt, Key::Escape);
    assert_eq!(rt.shell_request, Some(rune::Shell::Quit));
    let mut rt = runtime();
    rt.update(|state, tx| state.finish_launch(Ok(()), tx));
    assert_eq!(rt.shell_request, Some(rune::Shell::Quit));
    let mut rt = runtime();
    rt.update(|state, tx| state.finish_launch(Err("failed".into()), tx));
    assert_eq!(rt.shell_request, None);
}
