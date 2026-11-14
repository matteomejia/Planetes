use crate::{state::State, visual::*};
use planetes_core::SourceGame;
use rune::{
    Cx, El, Key, KeyEvent, ScrollSpec, Spring, SurfaceFit, Tx, col, el, row, spacer, text,
    text_input,
};

const ICON: f32 = 20.0;
/// The results column; the preview panel takes the rest of the window.
const LIST_WIDTH: f32 = 360.0;

pub(crate) fn view(s: &State, _cx: &mut Cx<State>) -> El {
    let p = palette();
    let mut body = col()
        .w_full()
        .h_full()
        .fill(p.bg)
        .motion(Spring::NONE)
        .child(search(s))
        .child(hairline())
        .child(
            row()
                .w_full()
                .grow(1.0)
                .min_h(0.0)
                .child(results(s))
                .child(el().w(1.0).h_full().fill(p.line))
                .child(crate::preview::panel(s)),
        );
    if let Some(error) = &s.error {
        body = body.child(hairline()).child(
            row().w_full().px(18.0).py(8.0).role("alert").child(
                text(error.clone())
                    .wrap()
                    .key("launch-error")
                    .font(SMALL)
                    .color(p.error)
                    .grow(1.0)
                    .min_w(0.0),
            ),
        );
    }
    if let Some(notice) = &s.notice {
        body = body.child(hairline()).child(
            row().w_full().px(18.0).py(8.0).child(
                text(notice.clone())
                    .wrap()
                    .key("notice")
                    .font(SMALL)
                    .color(p.muted)
                    .grow(1.0)
                    .min_w(0.0),
            ),
        );
    }
    body.child(hairline()).child(status(s))
}

fn search(s: &State) -> El {
    let p = palette();
    let input = text_input(&s.query, QUERY)
        .key("search")
        .single_line()
        .grow(1.0)
        .min_w(0.0)
        .h(28.0)
        .color(p.ink)
        .caret(p.accent, p.accent.alpha(0.35))
        .placeholder("Search games", p.faint)
        .label("Search games")
        .on_edit(|state: &mut State| &mut state.query)
        .on_input_change(|state: &mut State, tx| {
            state.filter();
            if let Some(&index) = state.results.first() {
                tx.scroll_into_view(format!("game-{index}"));
            }
            tx.instant();
        })
        .on_input_key(|state: &mut State, event: &KeyEvent, tx: &mut Tx| {
            match event.key {
                Key::Up => state.move_selection(-1, tx),
                Key::Down => state.move_selection(1, tx),
                Key::Escape => state.dismiss(tx),
                _ => return false,
            }
            true
        })
        .on_submit(|state: &mut State, tx| {
            if !tx.key_repeat() {
                state.launch_selected();
            }
        });
    row()
        .w_full()
        .h(56.0)
        .px(18.0)
        .center_y()
        .window_drag()
        .child(input)
}

fn results(s: &State) -> El {
    let p = palette();
    let now = crate::history::now();
    let mut list = col().key("results").w(LIST_WIDTH).padding(6.0);
    for (position, &index) in s.results.iter().enumerate() {
        let game = &s.games[index];
        let clicked = game.clone();
        let selected = position == s.selected;
        let store = crate::preview::store_name(game.store);
        let details = details(game, now);
        let label = match &details {
            Some(details) => format!("Launch {} via {store}, {details}", game.title),
            None => format!("Launch {} via {store}", game.title),
        };
        let image = s.icons.get(&(game.store, game.source_id.clone()));
        let tile = match image.filter(|image| image.is_ready() && image.error().is_none()) {
            Some(image) => el()
                .size(ICON, ICON)
                .radius(3.0)
                .pixels_fit(image.pixels(), SurfaceFit::Fit),
            None => monogram(&game.title, ICON),
        };
        list = list.child(
            row()
                .key(format!("game-{index}"))
                .w_full()
                .h(36.0)
                .px(10.0)
                .gap(11.0)
                .center_y()
                .radius(4.0)
                .fill(if selected {
                    p.selected
                } else {
                    rune::Color::TRANSPARENT
                })
                .hover(p.hover)
                .motion(Spring::NONE)
                .focus_ring(1.5, 0.0, p.accent)
                .role("button")
                .label(label)
                .focusable(true)
                .child(tile)
                .child(
                    text(game.title.clone())
                        .truncate()
                        .font(BODY)
                        .color(p.ink)
                        .grow(1.0)
                        .min_w(0.0),
                )
                .child(text(store).font(SMALL).color(p.muted))
                .on_press_tx(move |state: &mut State, _| {
                    if !state.launching {
                        state.selected = position;
                        state.launch_game(clicked.clone());
                    }
                }),
        );
    }
    if s.results.is_empty() {
        list = list.child(empty_state(s));
    }
    list.scroll(ScrollSpec {
        stick_bottom: false,
    })
    .grow(1.0)
    .min_h(0.0)
}

/// "2 weeks ago · 48 h": when the game was last played and the store's recorded playtime,
/// read out with each row. The preview panel shows the same facts for the selected game.
pub(crate) fn details(game: &SourceGame, now: u64) -> Option<String> {
    use crate::preview::{ago, playtime};
    let played = game.last_played.map(|at| ago(now.saturating_sub(at)));
    let parts: Vec<_> = played
        .into_iter()
        .chain(game.playtime_minutes.map(playtime))
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn empty_state(s: &State) -> El {
    let p = palette();
    let message = if s.loading {
        "Scanning installed games…".to_string()
    } else if s.games.is_empty() {
        format!(
            "No installed {} games found. Install one, then press {}.",
            STORES, RESCAN_KEY
        )
    } else {
        format!("No installed games match “{}”.", s.query.text.trim())
    };
    row().key("empty-state").w_full().px(12.0).py(10.0).child(
        text(message)
            .wrap()
            .font(BODY)
            .color(p.muted)
            .grow(1.0)
            .min_w(0.0),
    )
}

fn status(s: &State) -> El {
    let p = palette();
    let count = if s.launching {
        "Opening…".to_string()
    } else if s.loading {
        "Scanning…".to_string()
    } else if s.query.text.trim().is_empty() {
        let n = s.results.len();
        format!("{n} {}", if n == 1 { "game" } else { "games" })
    } else {
        format!("{} of {}", s.results.len(), s.games.len())
    };
    let mut left = row()
        .gap(6.0)
        .center_y()
        .min_w(0.0)
        .child(text(count).key("count").font(SMALL).color(p.muted));
    if s.warnings > 0 {
        left = left.child(
            text("· Some games could not be checked")
                .key("scan-warning")
                .truncate()
                .font(SMALL)
                .color(p.faint)
                .min_w(0.0),
        );
    }
    let rescan = hint(RESCAN_KEY, "rescan")
        .key("refresh")
        .px(4.0)
        .h(22.0)
        .radius(3.0)
        .hover(p.hover)
        .focus_ring(1.5, 0.0, p.accent)
        .role("button")
        .label("Rescan games")
        .focusable(true)
        .opacity(if s.loading { 0.5 } else { 1.0 })
        .on_press_tx(|state: &mut State, _| state.refresh());
    row()
        .w_full()
        .h(30.0)
        .px(16.0)
        .gap(10.0)
        .center_y()
        .window_drag()
        .child(left)
        .child(spacer())
        .child(hint("Enter", "launch"))
        .child(rescan)
        .child(hint("Esc", "close"))
}

/// Rune's `Key::Command('r')` is Command on macOS and Ctrl elsewhere.
#[cfg(target_os = "macos")]
const RESCAN_KEY: &str = "⌘R";
#[cfg(not(target_os = "macos"))]
const RESCAN_KEY: &str = "Ctrl+R";
/// Epic isn't scanned on macOS yet.
#[cfg(target_os = "macos")]
const STORES: &str = "Steam";
#[cfg(not(target_os = "macos"))]
const STORES: &str = "Steam or Epic";

fn hint(key: &str, action: &str) -> El {
    let p = palette();
    row()
        .gap(4.0)
        .center_y()
        .child(text(key).font(SMALL_STRONG).color(p.muted))
        .child(text(action).font(SMALL).color(p.faint))
}
