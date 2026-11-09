//! The panel beside the results: the selected game's key art, logo and play details.
//! Art comes only from files the store already keeps (Steam's library cache); a game without
//! any shows its own icon over a blurred copy of it. Nothing is downloaded.
use crate::{state::State, visual::*};
use planetes_core::{SourceGame, StoreId};
use rune::{El, Image, SurfaceFit, col, el, row, text};

/// The art across the top of the panel, cropped to fill and fading into the background.
const ART_HEIGHT: f32 = 264.0;
const LOGO_MAX: (f32, f32) = (240.0, 84.0);
const ICON: f32 = 64.0;

type Key = (StoreId, String);

/// One game's images. Only the selected game's and the one still on screen are held: hero
/// art decodes to about 6 MB with mip levels, so two at most.
#[derive(Clone)]
pub(crate) struct Preview {
    key: Key,
    hero: Option<Image>,
    logo: Option<Image>,
    icon: Option<Image>,
}

impl Preview {
    pub(crate) fn key(&self) -> (StoreId, &str) {
        (self.key.0, &self.key.1)
    }

    /// The same game with the same images: newly downloaded art makes a different preview.
    fn same(&self, other: &Preview) -> bool {
        let ids = |p: &Preview| [&p.hero, &p.logo, &p.icon].map(|i| i.as_ref().map(Image::id));
        self.key == other.key && ids(self) == ids(other)
    }

    fn ready(&self) -> bool {
        [&self.hero, &self.logo, &self.icon]
            .into_iter()
            .flatten()
            .all(Image::is_ready)
    }
}

fn usable(image: &Option<Image>) -> Option<&Image> {
    image
        .as_ref()
        .filter(|image| image.is_ready() && image.error().is_none())
}

/// Request the selected game's art and switch the panel to it once decoded, so moving the
/// selection never flashes an empty panel. Returns whether the panel changed.
pub(crate) fn poll(state: &mut State) -> bool {
    let selected = state
        .results
        .get(state.selected)
        .map(|&index| &state.games[index]);
    let Some(game) = selected else {
        let changed = state.shown.is_some();
        state.preview = None;
        state.shown = None;
        return changed;
    };
    let key = (game.store, game.source_id.clone());
    if state.preview.as_ref().is_none_or(|p| p.key != key) {
        let Some(cache) = &state.image_cache else {
            return false;
        };
        state.preview = Some(Preview {
            key,
            hero: game
                .hero_path
                .as_ref()
                .map(|path| cache.file(path, 0.0, 1.0)),
            logo: game
                .logo_path
                .as_ref()
                .map(|path| cache.file(path, 0.0, 1.0)),
            icon: game
                .icon_path
                .as_ref()
                .map(|path| crate::state::icon_image(cache, path, ICON)),
        });
    }
    let preview = state.preview.as_ref().expect("requested above");
    if preview.ready() && state.shown.as_ref().is_none_or(|s| !s.same(preview)) {
        state.shown = Some(preview.clone());
        return true;
    }
    false
}

/// Decode the selected game's art before a headless snapshot draws.
pub(crate) fn settle(state: &mut State) {
    poll(state);
    if let Some(preview) = &state.preview {
        for image in [&preview.hero, &preview.logo, &preview.icon]
            .into_iter()
            .flatten()
        {
            let _ = image.wait();
        }
    }
    poll(state);
}

pub(crate) fn panel(s: &State) -> El {
    let p = palette();
    let body = col().grow(1.0).min_w(0.0).h_full();
    // Until the selected game's art decodes, the previous game stays up, details included.
    // With nothing decoded yet (the first summon), its details show over a plain backdrop.
    let find = |preview: &'_ Preview| {
        s.games
            .iter()
            .find(|g| (g.store, g.source_id.as_str()) == preview.key())
    };
    let chosen = [&s.shown, &s.preview]
        .into_iter()
        .flatten()
        .find_map(|preview| Some((find(preview)?, preview)));
    let Some((game, preview)) = chosen else {
        return body.key("preview");
    };
    // Lifted into the art's faded lower edge.
    let mut info = col().gap(8.0).w_full().px(20.0).mt(-34.0).child(
        text(game.title.clone())
            .key("preview-title")
            .wrap()
            .font(TITLE)
            .color(p.ink)
            .w_full(),
    );
    let now = crate::history::now();
    let facts = [
        Some(("Store", store_name(game.store).to_string())),
        Some((
            "Last played",
            game.last_played
                .map_or_else(|| "Not yet".into(), |at| ago(now.saturating_sub(at))),
        )),
        game.playtime_minutes.map(|m| ("Playtime", playtime(m))),
    ];
    for (label, value) in facts.into_iter().flatten() {
        info = info.child(
            row()
                .gap(12.0)
                .w_full()
                .child(text(label).font(SMALL).color(p.faint).w(78.0))
                .child(
                    text(value)
                        .font(BODY)
                        .color(p.ink)
                        .truncate()
                        .grow(1.0)
                        .min_w(0.0),
                ),
        );
    }
    body.key("preview")
        .label(format!("{} preview", game.title))
        .child(art(game, preview))
        .child(info)
}

fn art(game: &SourceGame, preview: &Preview) -> El {
    let p = palette();
    let frame = el()
        .key("preview-art")
        .w_full()
        .h(ART_HEIGHT)
        .clip_rounded(0.0);
    if !preview.ready() {
        return frame;
    }
    let backdrop = match (usable(&preview.hero), usable(&preview.icon)) {
        (Some(hero), _) => el()
            .w_full()
            .h_full()
            .pixels_fit(hero.pixels(), SurfaceFit::Fill),
        // No local art: the game's icon over a soft, blurred copy of itself.
        (None, Some(icon)) => el()
            .w_full()
            .h_full()
            .center()
            .child(
                el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
                    .pixels_fit(icon.pixels(), SurfaceFit::Fill)
                    .blur(28.0)
                    .opacity(0.35),
            )
            .child(
                el().size(ICON, ICON)
                    .radius(10.0)
                    .pixels_fit(icon.pixels(), SurfaceFit::Fit),
            ),
        (None, None) => el()
            .w_full()
            .h_full()
            .center()
            .radial_gradient(
                [0.5, 0.45],
                [320.0, 150.0],
                [(0.0, p.tile.alpha(0.9)), (1.0, p.tile.alpha(0.0))],
            )
            .child(monogram(&game.title, 48.0)),
    };
    let mut frame = frame.child(backdrop).child(
        el().pin(Some(0.0), None, Some(0.0), Some(0.0))
            .h(ART_HEIGHT * 0.6)
            // Seats the logo and title, and fades the art's lower edge into the panel.
            .linear_gradient(
                std::f32::consts::PI,
                [(0.0, p.bg.alpha(0.0)), (0.55, p.bg.alpha(0.7)), (1.0, p.bg)],
            ),
    );
    if usable(&preview.hero).is_some()
        && let Some(logo) = usable(&preview.logo)
        && let Some([w, h]) = logo.size()
    {
        let aspect = w as f32 / h.max(1) as f32;
        let width = LOGO_MAX.0.min(LOGO_MAX.1 * aspect);
        frame = frame.child(
            el().key("preview-logo")
                .pin(Some(20.0), None, None, Some(44.0))
                .size(width, width / aspect)
                .pixels_fit(logo.pixels(), SurfaceFit::Fit),
        );
    }
    frame
}

pub(crate) fn store_name(store: StoreId) -> &'static str {
    match store {
        StoreId::Steam => "Steam",
        StoreId::Epic => "Epic",
    }
}

pub(crate) fn playtime(minutes: u64) -> String {
    match minutes {
        0..60 => format!("{minutes} min"),
        _ => format!("{} h", (minutes + 30) / 60),
    }
}

pub(crate) fn ago(seconds: u64) -> String {
    const DAY: u64 = 24 * 60 * 60;
    let days = seconds / DAY;
    let (n, unit) = match days {
        0 => return "today".into(),
        1 => return "yesterday".into(),
        2..14 => (days, "day"),
        14..60 => (days / 7, "week"),
        60..730 => (days / 30, "month"),
        _ => (days / 365, "year"),
    };
    format!("{n} {unit}s ago")
}
