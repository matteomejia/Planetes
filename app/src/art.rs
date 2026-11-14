//! Key art a store serves only from its CDN (Epic), downloaded once into
//! `%LOCALAPPDATA%\Planetes\art` and used like Steam's local art from then on. Requests go
//! only to the store's own HTTPS CDN (the scanner checks), carry no account or key, and stop
//! with `"download_art": false` in settings.json. Steam art is never downloaded.
use crate::{background, state::State};
use planetes_core::SourceGame;
use rune::AppHandle;
use std::{
    collections::HashSet,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

/// Larger than the panel draws at 2x; Epic's CDN resizes on request, about 250 KB a game.
const HERO_WIDTH: u32 = 1280;
const LOGO_WIDTH: u32 = 640;
const MAX_BYTES: u64 = 8 * 1024 * 1024;

fn dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    let base = Path::new(&std::env::var_os("HOME")?).join("Library/Caches");
    #[cfg(not(target_os = "macos"))]
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    Some(base.join("Planetes").join("art"))
}

/// A stable file name per URL (FNV-1a), so a changed image on the CDN is a new file.
fn cached(url: &str) -> Option<PathBuf> {
    let hash = url.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    });
    Some(dir()?.join(format!("{hash:016x}.img")))
}

/// Use art already downloaded in an earlier session.
pub fn attach_cached(games: &mut [SourceGame]) {
    for game in games {
        let local = |url: &Option<String>| {
            url.as_deref()
                .and_then(cached)
                .filter(|path| path.is_file())
        };
        if game.hero_path.is_none() {
            game.hero_path = local(&game.hero_url);
        }
        if game.logo_path.is_none() {
            game.logo_path = local(&game.logo_url);
        }
    }
}

/// Download the art not cached yet, one image at a time in the background. Each URL is tried
/// once a session; a failure (offline, say) is retried after the next start.
pub fn fetch_missing(handle: AppHandle<State>, games: &[SourceGame], tried: &mut HashSet<String>) {
    if !enabled() {
        return;
    }
    let wanted: Vec<(String, u32)> = games
        .iter()
        .flat_map(|game| {
            [
                (game.hero_path.is_none(), &game.hero_url, HERO_WIDTH),
                (game.logo_path.is_none(), &game.logo_url, LOGO_WIDTH),
            ]
        })
        .filter_map(|(missing, url, width)| Some((url.clone().filter(|_| missing)?, width)))
        .filter(|(url, _)| tried.insert(url.clone()))
        .collect();
    if wanted.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build()
            .into();
        for (url, width) in wanted {
            if download(&agent, &url, width).is_ok() {
                handle.update(|state, _| state.art_downloaded());
            }
        }
    });
}

fn download(agent: &ureq::Agent, url: &str, width: u32) -> Result<(), String> {
    let path = cached(url).ok_or("no local app data folder")?;
    let separator = if url.contains('?') { '&' } else { '?' };
    let mut response = agent
        .get(&format!("{url}{separator}w={width}&resize=1"))
        .header("Accept", "image/jpeg,image/png")
        .call()
        .map_err(|error| error.to_string())?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_vec()
        .map_err(|error| error.to_string())?;
    if !is_image(&bytes) {
        return Err("not a JPEG or PNG image".into());
    }
    std::fs::create_dir_all(path.parent().ok_or("no art folder")?).map_err(|e| e.to_string())?;
    // Written whole, then renamed, so a reader never sees half an image.
    let partial = path.with_extension("part");
    std::fs::File::create(&partial)
        .and_then(|mut file| file.write_all(&bytes))
        .and_then(|()| std::fs::rename(&partial, &path))
        .map_err(|error| error.to_string())
}

pub(crate) fn is_image(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xff, 0xd8, 0xff]) || bytes.starts_with(b"\x89PNG\r\n\x1a\n")
}

/// `"download_art": false` in settings.json turns downloads off; already cached art stays.
fn enabled() -> bool {
    background::settings_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .is_none_or(|text| downloads_from_json(&text))
}

pub(crate) fn downloads_from_json(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|value| value.get("download_art")?.as_bool())
        .unwrap_or(true)
}

impl State {
    fn art_downloaded(&mut self) {
        attach_cached(&mut self.games);
        // Request the selected game's art again; the panel keeps what it shows meanwhile.
        self.preview = None;
    }
}
