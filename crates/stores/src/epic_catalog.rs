//! Art links from the Epic launcher's catalog cache, `Data/Catalog/catcache.bin` beside its
//! manifests: base64-encoded JSON with each item's key images on Epic's CDN. The images
//! themselves aren't stored locally; the app downloads them (discovery stays offline).

use std::{collections::HashMap, fs, io, path::Path};

use base64::Engine;
use planetes_core::{ScanReport, StoreId};
use serde::Deserialize;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Art {
    pub hero_url: Option<String>,
    pub logo_url: Option<String>,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(default, rename = "keyImages")]
    key_images: Vec<KeyImage>,
}

#[derive(Deserialize)]
struct KeyImage {
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

/// Art by catalog item ID. A missing cache is normal; a broken one warns.
pub(super) fn read(manifests: &Path, report: &mut ScanReport) -> HashMap<String, Art> {
    let Some(path) = manifests
        .parent()
        .map(|data| data.join("Catalog/catcache.bin"))
    else {
        return HashMap::new();
    };
    let text = match fs::read(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return HashMap::new(),
        Err(error) => {
            report.warn(StoreId::Epic, Some(path), error.to_string());
            return HashMap::new();
        }
    };
    match parse(&text) {
        Ok(art) => art,
        Err(error) => {
            report.warn(
                StoreId::Epic,
                Some(path),
                format!("Cannot read Epic's catalog cache: {error}"),
            );
            HashMap::new()
        }
    }
}

fn parse(text: &[u8]) -> Result<HashMap<String, Art>, String> {
    let json = base64::engine::general_purpose::STANDARD
        .decode(text.trim_ascii())
        .map_err(|error| error.to_string())?;
    let items: Vec<Item> = serde_json::from_slice(&json).map_err(|error| error.to_string())?;
    Ok(items
        .into_iter()
        .map(|item| {
            let find = |kinds: &[&str]| {
                kinds.iter().find_map(|kind| {
                    item.key_images
                        .iter()
                        .find(|image| image.kind == *kind && trusted(&image.url))
                        .map(|image| image.url.clone())
                })
            };
            let art = Art {
                // Wide key art, usually 16:9; cropped to fill the panel.
                hero_url: find(&["DieselGameBox", "OfferImageWide"]),
                logo_url: find(&["DieselGameBoxLogo"]),
            };
            (item.id, art)
        })
        .filter(|(_, art)| *art != Art::default())
        .collect())
}

/// Only Epic's own CDN over HTTPS: the app fetches these, and the file is not ours.
fn trusted(url: &str) -> bool {
    url.strip_prefix("https://")
        .and_then(|rest| rest.split(['/', '?', '#']).next())
        .is_some_and(|host| host == "epicgames.com" || host.ends_with(".epicgames.com"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_epic_hosts_are_trusted() {
        assert!(trusted("https://cdn1.epicgames.com/item/x"));
        assert!(!trusted("http://cdn1.epicgames.com/item/x"));
        assert!(!trusted("https://cdn1.epicgames.com.example.org/x"));
        assert!(!trusted("https://example.org/?epicgames.com"));
        assert!(!trusted("https://evil-epicgames.com/x"));
    }
}
