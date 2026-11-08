//! Background mode: Planetes stays resident, a global hotkey summons it, and a tray icon offers
//! rescan, settings and quit. Settings live in a small JSON file the user can edit; changes
//! apply without a restart.
use crate::{platform, state::State};
use rune::{AppHandle, Shortcut, ShortcutError, Tray, TrayEvent, TrayIcon, TrayItem, TraySpec};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

pub const DEFAULT_HOTKEY: &str = "alt+space";

/// `%APPDATA%\Planetes\settings.json` on Windows.
pub fn settings_path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("Planetes").join("settings.json"))
}

/// The configured hotkey, or the default when the file is missing. An unreadable file is an
/// error so a typo doesn't silently fall back to a key the user moved away from.
pub fn read_hotkey() -> Result<String, String> {
    let Some(path) = settings_path() else {
        return Ok(DEFAULT_HOTKEY.into());
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DEFAULT_HOTKEY.into());
        }
        Err(error) => return Err(format!("Could not read settings: {error}.")),
    };
    hotkey_from_json(&text)
}

pub(crate) fn hotkey_from_json(text: &str) -> Result<String, String> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| format!("settings.json is invalid: {error}."))?;
    match value.get("hotkey") {
        None => Ok(DEFAULT_HOTKEY.into()),
        Some(serde_json::Value::String(hotkey)) => Ok(hotkey.trim().to_lowercase()),
        Some(_) => {
            Err("settings.json: \"hotkey\" must be text, such as \"ctrl+alt+space\".".into())
        }
    }
}

/// "ctrl+alt+space" → "Ctrl+Alt+Space".
pub fn hotkey_label(hotkey: &str) -> String {
    hotkey
        .split('+')
        .map(|part| {
            let mut chars = part.trim().chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<Vec<_>>()
        .join("+")
}

impl State {
    /// Register the configured hotkey, replacing the previous one. On failure the previous
    /// registration is kept and the reason shows in the window and tray tooltip.
    pub fn apply_settings(&mut self) {
        let Some(handle) = self.handle.clone() else {
            return;
        };
        let hotkey = match read_hotkey() {
            Ok(hotkey) => hotkey,
            Err(error) => {
                self.notice = Some(error);
                return;
            }
        };
        if self
            .hotkey
            .as_ref()
            .is_some_and(|(_, current)| *current == hotkey)
        {
            self.notice = None;
            return;
        }
        let label = hotkey_label(&hotkey);
        let shortcut: Shortcut = match hotkey.parse() {
            Ok(shortcut) => shortcut,
            Err(_) => {
                self.notice = Some(format!(
                    "“{hotkey}” is not a valid hotkey in settings.json."
                ));
                return;
            }
        };
        if let Some((id, _)) = self.hotkey.take() {
            let _ = handle.unregister_shortcut(id);
        }
        match handle.register_shortcut(shortcut, |state: &mut State, tx| state.toggle(tx)) {
            Ok(id) => {
                self.hotkey = Some((id, hotkey));
                self.notice = None;
            }
            Err(error) => {
                self.notice = Some(match error {
                    ShortcutError::Conflict => format!(
                        "{label} is already used by another app. Choose another hotkey in settings."
                    ),
                    other => format!("Could not register {label}: {other}."),
                });
            }
        }
        self.update_tray();
    }

    pub fn hotkey_text(&self) -> Option<String> {
        self.hotkey.as_ref().map(|(_, hotkey)| hotkey_label(hotkey))
    }

    pub fn update_tray(&self) {
        let Some(tray) = &self.tray else {
            return;
        };
        tray.set_menu(menu());
        tray.set_tooltip(match self.hotkey_text() {
            Some(label) => format!("Planetes ({label})"),
            None => "Planetes".into(),
        });
    }

    fn on_tray(&mut self, event: TrayEvent, tx: &mut rune::Tx) {
        match event {
            TrayEvent::Activate { .. } => self.summon(),
            TrayEvent::Menu(id) => match id.as_str() {
                "open" => self.summon(),
                "rescan" => self.refresh(),
                "autostart" => {
                    if let Err(error) = platform::set_autostart(!platform::autostart_enabled()) {
                        self.notice = Some(error);
                    }
                    self.update_tray();
                }
                "settings" => open_settings(),
                "quit" => tx.quit(),
                _ => {}
            },
            _ => {}
        }
    }
}

fn menu() -> Vec<TrayItem> {
    let autostart = if platform::autostart_enabled() {
        "✓ Start with Windows"
    } else {
        "Start with Windows"
    };
    vec![
        TrayItem::action("open", "Open Planetes"),
        TrayItem::action("rescan", "Rescan games"),
        TrayItem::Separator,
        TrayItem::action("autostart", autostart),
        TrayItem::action("settings", "Edit settings…"),
        TrayItem::Separator,
        TrayItem::action("quit", "Quit Planetes"),
    ]
}

/// Create the settings file with its defaults if needed, then open it in Notepad, which every
/// Windows install has, rather than whatever `.json` happens to be associated with.
fn open_settings() {
    let Some(path) = settings_path() else {
        return;
    };
    if !path.exists() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let defaults = format!("{{\n  \"hotkey\": \"{DEFAULT_HOTKEY}\"\n}}\n");
        let _ = std::fs::write(&path, defaults);
    }
    let _ = std::process::Command::new("notepad.exe").arg(&path).spawn();
}

/// The tray icon. Without a tray (unsupported platform or a shell error), the hotkey and a
/// second launch still summon the window.
pub fn tray(handle: AppHandle<State>) -> Option<Tray> {
    let spec = TraySpec::new("io.github.matteomejia.planetes", "Planetes", tray_icon()?)
        .tooltip("Planetes")
        .menu(menu());
    Tray::new(spec, move |event| {
        handle.update(move |state, tx| state.on_tray(event, tx));
    })
    .ok()
}

/// The app icon box-filtered from 256 px to 32 px.
pub(crate) fn tray_icon() -> Option<TrayIcon> {
    const SIZE: usize = 32;
    let bytes: &[u8] = include_bytes!("../assets/planetes.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut pixels = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut pixels).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let (width, height) = (info.width as usize, info.height as usize);
    let step = width / SIZE;
    if step == 0 || height / SIZE != step {
        return None;
    }
    let mut out = vec![0u8; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            // Average in premultiplied alpha so transparent edges don't darken.
            let mut sum = [0u32; 4];
            for sy in 0..step {
                for sx in 0..step {
                    let i = ((y * step + sy) * width + x * step + sx) * 4;
                    let alpha = u32::from(pixels[i + 3]);
                    for c in 0..3 {
                        sum[c] += u32::from(pixels[i + c]) * alpha;
                    }
                    sum[3] += alpha;
                }
            }
            let o = (y * SIZE + x) * 4;
            for c in 0..3 {
                out[o + c] = sum[c].checked_div(sum[3]).unwrap_or(0) as u8;
            }
            out[o + 3] = (sum[3] / (step * step) as u32) as u8;
        }
    }
    Some(TrayIcon::rgba(SIZE as u32, SIZE as u32, out))
}

/// Reapply settings when the file changes, checked every two seconds.
pub fn watch_settings(handle: AppHandle<State>) {
    let Some(path) = settings_path() else {
        return;
    };
    let modified = move || std::fs::metadata(&path).and_then(|m| m.modified()).ok();
    std::thread::spawn(move || {
        let mut last: Option<SystemTime> = modified();
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let now = modified();
            if now != last {
                last = now;
                handle.update(|state, _| state.apply_settings());
            }
        }
    });
}
