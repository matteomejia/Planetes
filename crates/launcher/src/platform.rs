#[cfg(windows)]
pub(super) fn open_uri(uri: &str) -> Result<(), String> {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Com::{
            COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
        },
        UI::{
            Shell::{
                ASSOCF_IS_PROTOCOL, ASSOCSTR_COMMAND, AssocQueryStringW, SEE_MASK_FLAG_NO_UI,
                SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW,
            },
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    };
    let scheme: Vec<u16> = uri
        .split(':')
        .next()
        .unwrap_or("")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut command = vec![0u16; 32768];
    let mut length = command.len() as u32;
    // SAFETY: buffers are valid for the call. This reads the protocol association; it does not execute it.
    let association = unsafe {
        AssocQueryStringW(
            ASSOCF_IS_PROTOCOL,
            ASSOCSTR_COMMAND,
            scheme.as_ptr(),
            std::ptr::null(),
            command.as_mut_ptr(),
            &mut length,
        )
    };
    if association != 0 {
        return Err("The store launcher is unavailable. Install or repair its URI association, then try again.".into());
    }
    let uri: Vec<u16> = uri.encode_utf16().chain(Some(0)).collect();
    let verb: Vec<u16> = "open\0".encode_utf16().collect();
    // SAFETY: initialized on the calling worker thread; balanced below on every successful initialization.
    let initialized = unsafe {
        CoInitializeEx(
            std::ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        )
    };
    if initialized < 0 {
        return Err(format!(
            "Windows launch initialization failed (0x{:08X}).",
            initialized as u32
        ));
    }
    // SAFETY: all-zero fields are valid optional/default values for this Windows structure.
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    // Return errors to our window, and finish protocol activation before Planetes exits.
    info.fMask = SEE_MASK_FLAG_NO_UI | SEE_MASK_NOASYNC;
    info.lpVerb = verb.as_ptr();
    info.lpFile = uri.as_ptr();
    info.nShow = SW_SHOWNORMAL;
    // SAFETY: strings are NUL-terminated, and all pointers remain valid for this synchronous call.
    let result = unsafe { ShellExecuteExW(&mut info) };
    // SAFETY: read the thread-local error immediately, before releasing COM.
    let error = if result == 0 {
        unsafe { GetLastError() }
    } else {
        0
    };
    // SAFETY: same thread, balanced with successful CoInitializeEx above (including S_FALSE).
    unsafe {
        CoUninitialize();
    }
    if result != 0 {
        Ok(())
    } else {
        Err(match error {
            2 | 3 | 1155 => "The store launcher is unavailable. Install or repair its URI association, then try again.".to_owned(),
            5 => "Windows denied the launch request.".to_owned(),
            _ => format!("Windows could not hand off the launch request (code {error})."),
        })
    }
}

#[cfg(target_os = "macos")]
pub(super) fn open_uri(uri: &str) -> Result<(), String> {
    let status = std::process::Command::new("/usr/bin/open")
        .arg(uri)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("The store launch request failed: {status}"))
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub(super) fn open_uri(_: &str) -> Result<(), String> {
    Err("Launching is currently supported on Windows and macOS.".to_owned())
}
