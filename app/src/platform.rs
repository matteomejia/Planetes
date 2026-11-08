/// Native setup for the frameless launcher window: icon, rounded corners, system shadow and a
/// centred position on the monitor it opened on.
#[cfg(windows)]
pub(crate) fn setup_window(window: &rune::WindowHandle) {
    use raw_window_handle::RawWindowHandle;
    let _ = window.with_raw_handle(|handle| {
        if let RawWindowHandle::Win32(handle) = handle.as_raw() {
            let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
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

/// Centre horizontally on the window's monitor, a little above the vertical middle, as
/// launchers sit.
#[cfg(windows)]
unsafe fn center(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::{
        Foundation::RECT,
        Graphics::Gdi::{
            GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
        },
        UI::WindowsAndMessaging::{
            GetWindowRect, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
        },
    };
    // SAFETY: valid window handle; the out-structs are sized and initialised for the calls.
    unsafe {
        let mut window = std::mem::zeroed::<RECT>();
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
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
