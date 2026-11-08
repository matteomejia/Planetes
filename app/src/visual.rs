//! Small, app-owned visual vocabulary; no remote artwork or runtime theme dependencies.
use rune::{Color, El, TextStyle, el, text};

#[derive(Clone, Copy)]
pub struct Palette {
    pub ink: Color,
    pub muted: Color,
    pub faint: Color,
    pub accent: Color,
    pub bg: Color,
    pub selected: Color,
    pub hover: Color,
    pub line: Color,
    pub tile: Color,
    pub error: Color,
}

pub fn palette() -> &'static Palette {
    static PALETTE: std::sync::OnceLock<Palette> = std::sync::OnceLock::new();
    PALETTE.get_or_init(|| Palette {
        ink: hex("#e8e8e6"),
        muted: hex("#8c8c89"),
        faint: hex("#6c6c69"),
        accent: hex("#4c8dff"),
        bg: hex("#181818"),
        selected: hex("#1f2b40"),
        hover: Color::WHITE.alpha(0.035),
        line: hex("#2a2a2a"),
        tile: hex("#2e2e2d"),
        error: hex("#f28b82"),
    })
}

fn hex(value: &str) -> Color {
    rune::color::hex(value).expect("valid authored palette color")
}

pub const QUERY: TextStyle = TextStyle::new(20.0, 400.0).lh(28.0);
pub const BODY: TextStyle = TextStyle::new(14.0, 400.0).lh(20.0);
pub const SMALL: TextStyle = TextStyle::new(12.0, 400.0).lh(16.0);
pub const SMALL_STRONG: TextStyle = TextStyle::new(12.0, 500.0).lh(16.0);

/// Full-width 1px separator.
pub fn hairline() -> El {
    el().w_full().h(1.0).fill(palette().line)
}

/// Neutral stand-in for a game without a local icon: its first letter on a grey square.
pub fn monogram(title: &str, size: f32) -> El {
    let p = palette();
    let letter: String = title
        .chars()
        .find(|c| c.is_alphanumeric())
        .into_iter()
        .flat_map(char::to_uppercase)
        .collect();
    el().size(size, size)
        .center()
        .radius(3.0)
        .fill(p.tile)
        .child(
            text(letter)
                .font(TextStyle::new(11.0, 500.0))
                .color(p.muted),
        )
}
