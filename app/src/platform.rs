/// Native setup for the frameless launcher window, each time Rune creates it: icon, rounded
/// corners, system shadow and a centred position on the monitor under the pointer.
#[cfg(windows)]
pub(crate) fn setup_window(window: &rune::WindowHandle) {
    use raw_window_handle::RawWindowHandle;
    let _ = window.with_raw_handle(|handle| {
        if let RawWindowHandle::Win32(handle) = handle.as_raw() {
            let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            WINDOW.store(hwnd as isize, std::sync::atomic::Ordering::Release);
            // SAFETY: this runs on the window thread with the window's own live handle.
            unsafe {
                set_icon(hwnd);
                round_and_shadow(hwnd);
                center(hwnd);
            }
        }
    });
}

#[cfg(not(windows))]
pub(crate) fn setup_window(_: &rune::WindowHandle) {}

#[cfg(windows)]
unsafe fn set_icon(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_SHARED, LoadImageW, SendMessageW, WM_SETICON,
        },
    };
    // SAFETY: resource 1 is embedded by build.rs; LR_SHARED gives the module ownership, so the
    // handles live as long as the executable.
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        for (which, size) in [(ICON_SMALL, 16), (ICON_BIG, 32)] {
            let icon = LoadImageW(
                module,
                std::ptr::without_provenance::<u16>(1), // Win32 integer resource ID, not a dereferenced pointer
                IMAGE_ICON,
                size,
                size,
                LR_SHARED,
            );
            if !icon.is_null() {
                SendMessageW(hwnd, WM_SETICON, which as usize, icon as isize);
            }
        }
    }
}

/// Windows 11 rounds and shadows a borderless window only when asked. Earlier Windows ignores
/// the corner attribute and keeps square corners.
#[cfg(windows)]
unsafe fn round_and_shadow(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::{
        Graphics::Dwm::{
            DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmExtendFrameIntoClientArea,
            DwmSetWindowAttribute,
        },
        UI::Controls::MARGINS,
    };
    let corner = DWMWCP_ROUND;
    // A one-pixel frame margin gives the popup DWM's standard window shadow.
    let margins = MARGINS {
        cxLeftWidth: 1,
        cxRightWidth: 1,
        cyTopHeight: 1,
        cyBottomHeight: 1,
    };
    // SAFETY: valid window handle; the attribute value outlives the call.
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&raw const corner).cast(),
            size_of_val(&corner) as u32,
        );
        DwmExtendFrameIntoClientArea(hwnd, &margins);
    }
}

/// Centre horizontally on the monitor under the pointer, a little above the vertical middle,
/// as launchers sit. A summoned launcher opens where the user is looking.
#[cfg(windows)]
unsafe fn center(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint},
        UI::WindowsAndMessaging::{
            GetCursorPos, GetWindowRect, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
        },
    };
    // SAFETY: valid window handle; the out-structs are sized and initialised for the calls.
    unsafe {
        let mut window = std::mem::zeroed::<RECT>();
        let mut cursor = POINT { x: 0, y: 0 };
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        GetCursorPos(&mut cursor);
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
        if GetWindowRect(hwnd, &mut window) == 0 || GetMonitorInfoW(monitor, &mut info) == 0 {
            return;
        }
        let work = info.rcWork;
        let (w, h) = (window.right - window.left, window.bottom - window.top);
        let x = work.left + (work.right - work.left - w) / 2;
        let y = work.top + ((work.bottom - work.top - h) * 2 / 5).max(0);
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}

/// The launcher window's handle, recorded each time Rune creates it.
#[cfg(windows)]
static WINDOW: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// Move the (usually hidden) window to the monitor under the pointer before it is shown.
#[cfg(windows)]
pub(crate) fn place_for_summon() {
    let hwnd = WINDOW.load(std::sync::atomic::Ordering::Acquire);
    if hwnd != 0 {
        // SAFETY: the handle was recorded from Rune's live window on this (the app) thread;
        // Win32 calls on a destroyed handle fail harmlessly.
        unsafe { center(hwnd as windows_sys::Win32::Foundation::HWND) }
    }
}

#[cfg(not(windows))]
pub(crate) fn place_for_summon() {}

#[cfg(windows)]
mod autostart {
    use windows_sys::Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW,
            RegSetKeyValueW,
        },
    };
    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE: &str = "Planetes";

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    pub fn enabled() -> bool {
        let (key, value) = (wide(RUN), wide(VALUE));
        // SAFETY: null-terminated strings; no output buffer requested.
        unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) == ERROR_SUCCESS
        }
    }

    pub fn set(on: bool) -> Result<(), String> {
        let (key, value) = (wide(RUN), wide(VALUE));
        let status = if on {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let command = wide(&format!("\"{}\" --background", exe.display()));
            // SAFETY: null-terminated strings; the data length covers the terminator.
            unsafe {
                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    key.as_ptr(),
                    value.as_ptr(),
                    REG_SZ,
                    command.as_ptr().cast(),
                    (command.len() * 2) as u32,
                )
            }
        } else {
            // SAFETY: null-terminated strings.
            unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), value.as_ptr()) }
        };
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!(
                "Windows refused the startup setting (error {status})."
            ))
        }
    }
}

/// Whether Planetes starts, hidden, when the user signs in.
#[cfg(windows)]
pub(crate) fn autostart_enabled() -> bool {
    autostart::enabled()
}
#[cfg(windows)]
pub(crate) fn set_autostart(on: bool) -> Result<(), String> {
    autostart::set(on)
}
#[cfg(not(windows))]
pub(crate) fn autostart_enabled() -> bool {
    false
}
#[cfg(not(windows))]
pub(crate) fn set_autostart(_: bool) -> Result<(), String> {
    Err("Starting with the system is only supported on Windows.".into())
}
