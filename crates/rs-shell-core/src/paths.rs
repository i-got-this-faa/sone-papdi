use std::path::PathBuf;

/// Return the XDG user runtime directory (`/run/user/<uid>`).
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir);
    }
    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/run/user/{}", uid))
}

/// Main daemon socket (subscribable API).
pub fn daemon_socket() -> PathBuf {
    runtime_dir().join("rs-shell.sock")
}

/// Clipboard daemon socket (rsclip compatibility).
pub fn clipboard_socket() -> PathBuf {
    runtime_dir().join("rs-shell-clipboard.sock")
}

/// Bar process activation socket.
pub fn bar_socket() -> PathBuf {
    runtime_dir().join("rs-shell-bar.sock")
}

/// User data directory for rs-shell.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("rs-shell");
    }
    dirs_next::data_dir()
        .map(|p| p.join("rs-shell"))
        .unwrap_or_else(|| PathBuf::from("/tmp/rs-shell-data"))
}

/// User config directory for rs-shell.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("rs-shell");
    }
    dirs_next::config_dir()
        .map(|p| p.join("rs-shell"))
        .unwrap_or_else(|| PathBuf::from("/tmp/rs-shell-config"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a test with a temporary XDG_RUNTIME_DIR value, restoring the original after.
    fn with_runtime_dir<T>(val: Option<&str>, f: impl FnOnce() -> T) -> T {
        let prev = std::env::var("XDG_RUNTIME_DIR").ok();
        match val {
            Some(v) => std::env::set_var("XDG_RUNTIME_DIR", v),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }
        let result = f();
        match prev {
            Some(v) => std::env::set_var("XDG_RUNTIME_DIR", v),
            None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }
        result
    }

    #[test]
    fn runtime_dir_under_run_user() {
        with_runtime_dir(None, || {
            let path = runtime_dir();
            assert!(path.to_string_lossy().starts_with("/run/user/"));
        });
    }

    #[test]
    fn sockets_are_in_runtime_dir() {
        with_runtime_dir(None, || {
            let rt = runtime_dir();
            assert_eq!(daemon_socket(), rt.join("rs-shell.sock"));
            assert_eq!(clipboard_socket(), rt.join("rs-shell-clipboard.sock"));
            assert_eq!(bar_socket(), rt.join("rs-shell-bar.sock"));
        });
    }

    #[test]
    fn xdg_runtime_dir_respected() {
        with_runtime_dir(Some("/tmp/custom-runtime"), || {
            assert_eq!(runtime_dir(), PathBuf::from("/tmp/custom-runtime"));
        });
    }
}
