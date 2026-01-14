//! Process Hardening
//! 
//! Implements OS-level security best practices to protect the agency process.
//! Derived from codex-rs patterns.

use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

/// Perform various process hardening steps:
/// - disabling core dumps
/// - disabling ptrace attach on macOS.
/// - removing dangerous environment variables such as DYLD_*
pub fn apply_hardening() {
    #[cfg(target_os = "macos")]
    hardening_macos();

    #[cfg(target_os = "linux")]
    hardening_linux();
}

#[cfg(target_os = "macos")]
fn hardening_macos() {
    // Prevent debuggers from attaching to this process.
    // libc::ptrace(libc::PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0);
    // Note: PT_DENY_ATTACH can be aggressive for development, 
    // but it's a standard SOTA practice for secure agents.
    unsafe {
        let _ = libc::ptrace(31, 0, std::ptr::null_mut(), 0); // 31 is PT_DENY_ATTACH
    }

    // Set the core file size limit to 0 to prevent core dumps.
    set_core_limit_to_zero();

    // Remove DYLD_ environment variables
    clear_env_vars_with_prefix(b"DYLD_");
}

#[cfg(target_os = "linux")]
fn hardening_linux() {
    // Disable ptrace attach
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
    }

    set_core_limit_to_zero();
    clear_env_vars_with_prefix(b"LD_");
}

#[cfg(unix)]
fn set_core_limit_to_zero() {
    let rlim = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe {
        libc::setrlimit(libc::RLIMIT_CORE, &rlim);
    }
}

#[cfg(unix)]
fn clear_env_vars_with_prefix(prefix: &[u8]) {
    let keys: Vec<OsString> = std::env::vars_os()
        .filter_map(|(key, _)| {
            if key.as_os_str().as_bytes().starts_with(prefix) {
                Some(key)
            } else {
                None
            }
        })
        .collect();

    for key in keys {
        std::env::remove_var(key);
    }
}