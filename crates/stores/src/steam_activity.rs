//! Last played and playtime from Steam's per-account `userdata/<id>/config/localconfig.vdf`.

use std::{collections::HashMap, fs, path::Path};

use planetes_core::{ScanReport, StoreId};
use steam_vdf_parser::{Obj, Value, parse_text};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Activity {
    pub last_played: Option<u64>,
    pub playtime_minutes: Option<u64>,
}

/// Activity by app ID. With several accounts on one PC, each game takes the activity of the
/// account that played it last. A missing or broken file only means no details.
pub(super) fn read(root: &Path, report: &mut ScanReport) -> HashMap<u32, Activity> {
    let mut result: HashMap<u32, Activity> = HashMap::new();
    let Ok(accounts) = fs::read_dir(root.join("userdata")) else {
        return result;
    };
    for account in accounts.flatten() {
        let path = account.path().join("config/localconfig.vdf");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let apps = match parse(&text) {
            Ok(apps) => apps,
            Err(error) => {
                report.warn(
                    StoreId::Steam,
                    Some(path),
                    format!("Cannot read play history: {error}"),
                );
                continue;
            }
        };
        for (id, activity) in apps {
            let entry = result.entry(id).or_default();
            if activity.last_played >= entry.last_played {
                *entry = activity;
            }
        }
    }
    result
}

fn parse(text: &str) -> Result<HashMap<u32, Activity>, String> {
    let vdf = parse_text(text).map_err(|error| error.to_string())?;
    let mut result = HashMap::new();
    // Steam has written both "apps" and "Apps" here over the years.
    let Some(apps) = ["Software", "Valve", "Steam", "apps"]
        .iter()
        .try_fold(vdf.value(), |value, key| child(value.as_obj()?, key))
        .and_then(Value::as_obj)
    else {
        return Ok(result);
    };
    for (id, app) in apps.iter() {
        let (Ok(id), Some(app)) = (id.parse(), app.as_obj()) else {
            continue;
        };
        let number = |key| {
            child(app, key)
                .and_then(Value::as_str)
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|&value| value > 0)
        };
        let activity = Activity {
            last_played: number("LastPlayed"),
            playtime_minutes: number("Playtime"),
        };
        if activity != Activity::default() {
            result.insert(id, activity);
        }
    }
    Ok(result)
}

fn child<'a, 'text>(obj: &'a Obj<'text>, key: &str) -> Option<&'a Value<'text>> {
    obj.iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}
