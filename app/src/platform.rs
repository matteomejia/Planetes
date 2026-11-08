#[cfg(windows)]
pub(crate) fn set_icon(window: &rune::WindowHandle) {
    use raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_SHARED, LoadImageW, SendMessageW, WM_SETICON,
        },
    };
    let _ = window.with_raw_handle(|handle| {
        if let RawWindowHandle::Win32(handle) = handle.as_raw() {
            // SAFETY: this runs on the window thread. Resource 1 is embedded by build.rs;
            // LR_SHARED gives the module ownership, so the handles live as long as the executable.
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
                        SendMessageW(
                            handle.hwnd.get() as _,
                            WM_SETICON,
                            which as usize,
                            icon as isize,
                        );
                    }
                }
            }
        }
    });
}

#[cfg(not(windows))]
pub(crate) fn set_icon(_: &rune::WindowHandle) {}
