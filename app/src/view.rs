use crate::{state::State, visual::*};
use planetes_core::StoreId;
use rune::{
    Color, Cx, El, Key, KeyEvent, ScrollSpec, Spring, SurfaceFit, TextStyle, Tx, col, el, row,
    spacer, text, text_input,
};

pub(crate) fn view(s: &State, cx: &mut Cx<State>) -> El {
    let p = palette();
    let motion = if cx.is_reduced_motion() {
        Spring::NONE
    } else {
        Spring::new(0.16, 0.0)
    };
    let mark = el()
        .size(30.0, 30.0)
        .center()
        .radius(9.0)
        .continuous_corners()
        .fill(p.accent.alpha(0.10))
        .border_gradient(1.0, p.accent.alpha(0.25), p.accent.alpha(0.05))
        .child(glyph(&ORBIT, 21.0, p.accent));
    let header = row()
        .w_full()
        .h(32.0)
        .gap(9.0)
        .center_y()
        .child(mark)
        .child(
            text("Planetes")
                .font(TextStyle::new(14.0, 600.0).tracking(-0.3))
                .color(p.ink),
        )
        .child(spacer())
        .child(
            row()
                .key("refresh")
                .h(28.0)
                .px(8.0)
                .gap(6.0)
                .center_y()
                .radius(7.0)
                .hover(Color::WHITE.alpha(0.04))
                .press(Color::WHITE.alpha(0.06))
                .hover_motion(motion, motion)
                .focus_ring(1.5, 1.0, p.accent)
                .role("button")
                .label("Refresh games")
                .focusable(true)
                .opacity(if s.loading { 0.5 } else { 1.0 })
                .child(glyph(&REFRESH, 13.0, p.muted))
                .child(
                    text(if s.loading { "Refreshing" } else { "Refresh" })
                        .font(SMALL)
                        .color(p.muted),
                )
                .on_press_tx(|state: &mut State, _| state.refresh()),
        );
    let input = text_input(&s.query, TextStyle::new(16.0, 400.0).lh(26.0))
        .key("search")
        .single_line()
        .grow(1.0)
        .min_w(0.0)
        .h(28.0)
        .color(p.ink)
        .placeholder("Find a game…", p.faint)
        .label("Find a game")
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
                Key::Escape => tx.quit(),
                _ => return false,
            }
            true
        })
        .on_submit(|state: &mut State, tx| {
            if !tx.key_repeat() {
                state.launch_selected();
            }
        });
    let search = row()
        .w_full()
        .h(56.0)
        .px(15.0)
        .gap(12.0)
        .center_y()
        .radius(12.0)
        .continuous_corners()
        .fill(p.panel)
        .gradient(Color::WHITE.alpha(0.015), Color::TRANSPARENT)
        .border_gradient(1.0, Color::WHITE.alpha(0.11), Color::WHITE.alpha(0.055))
        .focus_within_ring(1.0, 0.0, p.accent.alpha(0.45))
        .shadow(3.0, 14.0, Color::BLACK.alpha(0.12))
        .child(glyph(&SEARCH, 19.0, p.muted))
        .child(input);
    let mut results = col().key("results").w_full().gap(4.0).px(2.0).py(2.0);
    for (position, &index) in s.results.iter().enumerate() {
        let game = &s.games[index];
        let clicked = game.clone();
        let selected = position == s.selected;
        let store = match game.store {
            StoreId::Steam => "Steam",
            StoreId::Epic => "Epic Games",
        };
        let image = s.icons.get(&(game.store, game.source_id.clone()));
        let tile = match image.filter(|image| image.is_ready() && image.error().is_none()) {
            Some(image) => el()
                .size(32.0, 32.0)
                .radius(8.0)
                .pixels_fit(image.pixels(), SurfaceFit::Fit),
            None => monogram(&game.title),
        };
        let details = col()
            .grow(1.0)
            .min_w(0.0)
            .gap(1.0)
            .child(text(game.title.clone()).truncate().font(BODY).color(p.ink))
            .child(
                text(store)
                    .font(TextStyle::new(10.5, 400.0).lh(14.0))
                    .color(p.muted),
            );
        let enter = el()
            .size(26.0, 24.0)
            .center()
            .radius(6.0)
            .fill(if selected {
                p.accent.alpha(0.10)
            } else {
                Color::TRANSPARENT
            })
            .child(glyph(
                &RETURN,
                14.0,
                if selected {
                    p.accent
                } else {
                    Color::TRANSPARENT
                },
            ));
        results = results.child(
            row()
                .key(format!("game-{index}"))
                .w_full()
                .h(50.0)
                .px(11.0)
                .gap(12.0)
                .center_y()
                .radius(11.0)
                .continuous_corners()
                .fill(if selected {
                    p.panel
                } else {
                    Color::TRANSPARENT
                })
                .gradient(
                    if selected {
                        p.accent.alpha(0.09)
                    } else {
                        Color::TRANSPARENT
                    },
                    if selected {
                        p.accent.alpha(0.025)
                    } else {
                        Color::TRANSPARENT
                    },
                )
                .border_gradient(
                    1.0,
                    if selected {
                        p.accent.alpha(0.28)
                    } else {
                        Color::TRANSPARENT
                    },
                    if selected {
                        p.accent.alpha(0.10)
                    } else {
                        Color::TRANSPARENT
                    },
                )
                .hover(Color::WHITE.alpha(0.035))
                .press(p.accent.alpha(0.07))
                .motion(motion)
                .hover_motion(motion, motion)
                .focus_ring(1.5, 0.0, p.accent)
                .role("button")
                .label(format!("Launch {} via {store}", game.title))
                .focusable(true)
                .child(tile)
                .child(details)
                .child(enter)
                .on_press_tx(move |state: &mut State, _| {
                    if !state.launching {
                        state.selected = position;
                        state.launch_game(clicked.clone());
                    }
                }),
        );
    }
    if s.results.is_empty() {
        results = results.child(empty_state(s));
    }
    let count = if s.launching {
        "Opening…".into()
    } else if s.loading {
        "Finding games…".into()
    } else {
        format!(
            "{} {}",
            s.results.len(),
            if s.results.len() == 1 {
                "game"
            } else {
                "games"
            }
        )
    };
    let section = row()
        .w_full()
        .px(3.0)
        .center_y()
        .child(
            text(if s.query.text.trim().is_empty() {
                "YOUR LIBRARY"
            } else {
                "SEARCH RESULTS"
            })
            .font(TextStyle::new(9.0, 600.0).tracking(1.1))
            .color(p.faint),
        )
        .child(spacer())
        .child(text(count).key("count").font(SMALL).color(p.muted));
    let library = col()
        .w_full()
        .grow(1.0)
        .min_h(0.0)
        .gap(8.0)
        .child(section)
        .child(
            results
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .grow(1.0)
                .min_h(0.0),
        );
    let mut body = col()
        .w_full()
        .h_full()
        .padding(20.0)
        .gap(15.0)
        .fill(p.bg)
        .gradient(p.accent.alpha(0.025), Color::TRANSPARENT)
        .motion(Spring::NONE)
        .child(header)
        .child(search)
        .child(library);
    if let Some(error) = &s.error {
        let tint = p.error;
        body = body.child(
            row()
                .w_full()
                .gap(9.0)
                .padding(10.0)
                .radius(9.0)
                .fill(tint.alpha(0.055))
                .border(1.0, tint.alpha(0.13))
                .role("alert")
                .child(glyph(&ALERT, 15.0, tint))
                .child(
                    text(error.clone())
                        .wrap()
                        .key("launch-error")
                        .font(SMALL)
                        .color(tint)
                        .grow(1.0)
                        .min_w(0.0),
                ),
        );
    }
    if s.warnings > 0 {
        body = body.child(
            text("Some games could not be checked. Try Refresh.")
                .key("scan-warning")
                .font(SMALL)
                .color(p.muted),
        );
    }
    body.child(el().w_full().h(1.0).fill(p.line))
        .child(footer())
}

fn footer() -> El {
    let p = palette();
    row()
        .w_full()
        .center_y()
        .gap(5.0)
        .child(keycap("↑"))
        .child(keycap("↓"))
        .child(text("Navigate").font(SMALL).color(p.faint))
        .child(spacer())
        .child(keycap("↵"))
        .child(text("Launch").font(SMALL).color(p.muted))
        .child(el().w(8.0))
        .child(keycap("esc"))
        .child(text("Close").font(SMALL).color(p.faint))
}

fn empty_state(s: &State) -> El {
    let p = palette();
    let (title, detail) = if s.loading {
        ("Finding your games", "Checking your installed libraries…")
    } else if s.games.is_empty() {
        (
            "Your next game starts here",
            "Install a game in Steam or Epic, then refresh.",
        )
    } else {
        (
            "No matches this time",
            "Try a shorter name or an abbreviation.",
        )
    };
    col()
        .key("empty-state")
        .w_full()
        .py(27.0)
        .gap(8.0)
        .center()
        .child(
            el().size(42.0, 42.0)
                .center()
                .radius(13.0)
                .fill(p.accent.alpha(0.055))
                .child(glyph(
                    if s.loading { &ORBIT } else { &SEARCH },
                    22.0,
                    p.accent.alpha(0.65),
                )),
        )
        .child(text(title).font(BODY).color(p.ink))
        .child(text(detail).font(SMALL).color(p.muted))
}
