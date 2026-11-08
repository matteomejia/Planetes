use std::{collections::HashMap, fs, io, path::Path};

use planetes_core::{ScanReport, StoreId};

pub(super) struct Metadata {
    pub kind: Option<String>,
    pub icon_hash: Option<String>,
}

pub(super) fn read(root: &Path, report: &mut ScanReport) -> HashMap<u32, Metadata> {
    let path = root.join("appcache/appinfo.vdf");
    let data = match fs::read(&path) {
        Ok(data) => data,
        // A cache is optional. Never hide a game solely because metadata is unavailable.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return HashMap::new(),
        Err(error) => {
            report.warn(StoreId::Steam, Some(path), error.to_string());
            return HashMap::new();
        }
    };
    match parse(&data) {
        Ok(types) => types,
        Err(error) => {
            report.warn(
                StoreId::Steam,
                Some(path),
                format!("Cannot classify Steam apps: {error}"),
            );
            HashMap::new()
        }
    }
}

fn parse(data: &[u8]) -> Result<HashMap<u32, Metadata>, String> {
    // 0.1.2 checks for a 68-byte record header before reading the four-byte zero sentinel.
    // Pad only our in-memory copy so valid short EOF sentinels work in both cache versions.
    let mut padded = data.to_vec();
    padded.extend([0; 64]);
    // A changing/corrupt optional cache must not abort discovery. The parser can panic on a
    // truncated record size; retain manifest discovery if that happens.
    let vdf = std::panic::catch_unwind(|| steam_vdf_parser::parse_appinfo(&padded))
        .map_err(|_| "Malformed Steam metadata record".to_owned())?
        .map_err(|error| error.to_string())?;
    let mut result = HashMap::new();
    if let Some(apps) = vdf.as_obj() {
        for (id, app) in apps.iter() {
            if let Ok(id) = id.parse() {
                result.insert(
                    id,
                    Metadata {
                        kind: app
                            .get_str(&["appinfo", "common", "type"])
                            .map(str::to_ascii_lowercase),
                        icon_hash: app
                            .get_str(&["appinfo", "common", "clienticon"])
                            .or_else(|| app.get_str(&["appinfo", "common", "icon"]))
                            .filter(|hash| {
                                hash.len() == 40 && hash.bytes().all(|b| b.is_ascii_hexdigit())
                            })
                            .map(str::to_owned),
                    },
                );
            }
        }
    }
    Ok(result)
}
