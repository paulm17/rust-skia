//! Port of `src/utils/SkGetExecutablePath.h` and `src/utils/SkGetExecutablePath_mac.cpp`.

/// Returns a fully-qualified path to the currently-running executable.
#[cfg(target_os = "macos")]
pub fn sk_get_executable_path() -> String {
    use std::ffi::c_char;

    unsafe extern "C" {
        fn _NSGetExecutablePath(buf: *mut c_char, bufsize: *mut u32) -> i32;
    }

    let mut size: u32 = 0;
    // SAFETY: passing a null buffer asks dyld for the required size.
    unsafe {
        _NSGetExecutablePath(std::ptr::null_mut(), &mut size);
    }

    let mut result = vec![0u8; size as usize];
    // SAFETY: `result` has the size dyld reported (and dyld updates `size`).
    let rc = unsafe { _NSGetExecutablePath(result.as_mut_ptr() as *mut c_char, &mut size) };
    if rc != 0 {
        result.clear();
    }
    // The buffer is NUL-terminated when the call succeeds.
    if let Some(pos) = result.iter().position(|&b| b == 0) {
        result.truncate(pos);
    }
    String::from_utf8_lossy(&result).into_owned()
}

/// Non-macOS fallback; the macOS target is the only supported one (see `plan/MACOS-SCOPE.md`).
#[cfg(not(target_os = "macos"))]
pub fn sk_get_executable_path() -> String {
    String::new()
}
