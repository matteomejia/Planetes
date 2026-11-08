//! Small, app-owned visual vocabulary; no remote artwork or runtime theme dependencies.
use rune::{Color, El, Icon, TextStyle, el, icon, text};

#[derive(Clone, Copy)]
pub struct Palette {
    pub ink: Color,
    pub muted: Color,
    pub faint: Color,
    pub accent: Color,
    pub bg: Color,
    pub panel: Color,
    pub line: Color,
    pub error: Color,
}

pub fn palette() -> &'static Palette {
    static PALETTE: std::sync::OnceLock<Palette> = std::sync::OnceLock::new();
    PALETTE.get_or_init(|| Palette {
        ink: hex("#f0f0f8"),
        muted: hex("#a0a1b0"),
        faint: hex("#7e8094"),
        accent: hex("#ad9bff"),
        bg: hex("#101116"),
        panel: hex("#191a23"),
        line: Color::WHITE.alpha(0.055),
        error: hex("#fba0a5"),
    })
}

fn hex(value: &str) -> Color {
    rune::color::hex(value).expect("valid authored palette color")
}
pub const SMALL: TextStyle = TextStyle::new(11.0, 500.0).lh(16.0);
pub const BODY: TextStyle = TextStyle::new(14.0, 500.0).lh(20.0);

pub static ORBIT: Icon = Icon {
    name: "planetes-orbit",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="5"/><ellipse cx="12" cy="12" rx="11" ry="4" transform="rotate(-35 12 12)"/></g><circle cx="19.5" cy="5.8" r="1.7" fill="currentColor"/>"#,
};
pub static SEARCH: Icon = Icon {
    name: "search",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 4.5 4.5"/></g>"#,
};
pub static REFRESH: Icon = Icon {
    name: "refresh",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M20 10a8 8 0 0 0-14-3L3 10m0-6v6h6M4 14a8 8 0 0 0 14 3l3-3m0 6v-6h-6"/></g>"#,
};
pub static RETURN: Icon = Icon {
    name: "return",
    view: 24.0,
    body: r#"<path d="M20 5v8H5m5-5-5 5 5 5" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/>"#,
};
pub static ALERT: Icon = Icon {
    name: "alert",
    view: 24.0,
    body: r#"<g fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"><circle cx="12" cy="12" r="9"/><path d="M12 7v6m0 4v.1"/></g>"#,
};

pub fn glyph(icon_: &'static Icon, size: f32, color: Color) -> El {
    icon(icon_, size).color(color)
}

pub fn keycap(label: &str) -> El {
    let p = palette();
    el().min_w(20.0)
        .h(20.0)
        .px(5.0)
        .center()
        .radius(5.0)
        .fill(Color::WHITE.alpha(0.025))
        .border(1.0, Color::WHITE.alpha(0.08))
        .child(text(label).font(TextStyle::new(10.0, 500.0)).color(p.muted))
}

pub fn monogram(title: &str) -> El {
    let p = palette();
    let hash = title.bytes().fold(0u32, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(byte as u32)
    });
    let tint = match hash % 4 {
        0 => p.accent,
        1 => hex("#6bb8c2"),
        2 => hex("#d9a173"),
        _ => hex("#c088b3"),
    };
    let letters: String = title
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect();
    el().size(32.0, 32.0)
        .center()
        .radius(9.0)
        .continuous_corners()
        .fill(tint.alpha(0.10))
        .gradient(tint.alpha(0.09), Color::TRANSPARENT)
        .border_gradient(1.0, tint.alpha(0.24), tint.alpha(0.08))
        .child(text(letters).font(TextStyle::new(11.0, 600.0)).color(tint))
}
