use std::env;
use std::path::{Path, PathBuf};

fn has_bundled_assets(root: &Path) -> bool {
    root.join("assets").is_dir() && root.join("fonts").is_dir()
}

/// Dev: `--root` with assets/fonts. Installed: `../share/marius-agenda` or `/usr/share/marius-agenda`.
pub fn resolve_app_root(provided: &Path) -> PathBuf {
    if has_bundled_assets(provided) {
        return provided.canonicalize().unwrap_or_else(|_| provided.to_path_buf());
    }
    // Explicit `--root` (not ".") without assets: keep path; do not jump to /usr/share.
    if provided != Path::new(".") {
        return provided.canonicalize().unwrap_or_else(|_| provided.to_path_buf());
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(bin) = exe.parent() {
            let candidates = [
                bin.join("../share/marius-agenda"),
                PathBuf::from("/usr/share/marius-agenda"),
            ];
            for c in candidates {
                if has_bundled_assets(&c) {
                    return c.canonicalize().unwrap_or(c);
                }
            }
        }
    }
    provided.canonicalize().unwrap_or_else(|_| provided.to_path_buf())
}

/// `~/.local/share/marius-agenda/data` (or `$XDG_DATA_HOME/marius-agenda/data`).
pub fn default_user_data_dir() -> PathBuf {
    let base = env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".local/share"));
    base.join("marius-agenda").join("data")
}

/// User data: `MARIUS_AGENDA_DATA`, else `{root}/data` when assets ship with root, else XDG.
pub fn resolve_data_dir(root: &Path) -> PathBuf {
    if let Ok(p) = env::var("MARIUS_AGENDA_DATA") {
        return PathBuf::from(p);
    }
    if root.join("assets").is_dir() && root.join("fonts").is_dir() {
        return root.join("data");
    }
    default_user_data_dir()
}

/// Explicit data directory (e.g. tests) or env / `{root}/data`.
pub fn effective_data_dir(root: &Path, override_dir: Option<&Path>) -> PathBuf {
    override_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| resolve_data_dir(root))
}

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

pub fn home_dir() -> PathBuf {
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

/// User home for file dialogs (snap sets `HOME` under `~/snap/<app>/…`, not the real profile).
pub fn user_home_for_file_dialog() -> PathBuf {
    if let Ok(h) = env::var("SNAP_REAL_HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
}

/// Folder to open in GtkFileDialog — never a missing path (GTK/portals error otherwise).
pub fn picker_start_dir(last_output_dir: &str) -> PathBuf {
    let trimmed = last_output_dir.trim();
    if !trimmed.is_empty() {
        let p = PathBuf::from(trimmed);
        if p.is_dir() {
            return p;
        }
        if let Some(parent) = p.parent() {
            if parent.is_dir() {
                return parent.to_path_buf();
            }
        }
    }
    user_home_for_file_dialog()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_app_root_without_assets_keeps_provided_path() {
        let base = std::env::temp_dir().join(format!("marius-noroot-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let expected = base.canonicalize().unwrap_or_else(|_| base.clone());
        let root = resolve_app_root(&base);
        assert_eq!(root, expected);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn resolve_app_root_prefers_provided_when_assets_present() {
        let base = std::env::temp_dir().join(format!("marius-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("assets")).unwrap();
        fs::create_dir_all(base.join("fonts")).unwrap();
        assert!(has_bundled_assets(&resolve_app_root(&base)));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn picker_start_dir_uses_home_when_saved_path_missing() {
        let missing = home_dir().join("marius-picker-missing-test-dir");
        let start = picker_start_dir(missing.to_string_lossy().as_ref());
        assert!(start.is_dir());
        assert_eq!(start, user_home_for_file_dialog());
    }

    #[test]
    fn user_home_for_file_dialog_prefers_snap_real_home() {
        let prev_real = std::env::var("SNAP_REAL_HOME").ok();
        let prev_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("SNAP_REAL_HOME", "/tmp/marius-real-home-test");
            std::env::set_var("HOME", "/tmp/marius-snap-home-test");
        }
        assert_eq!(
            user_home_for_file_dialog(),
            PathBuf::from("/tmp/marius-real-home-test")
        );
        unsafe {
            if let Some(h) = prev_real {
                std::env::set_var("SNAP_REAL_HOME", h);
            } else {
                std::env::remove_var("SNAP_REAL_HOME");
            }
            if let Some(h) = prev_home {
                std::env::set_var("HOME", h);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn picker_start_dir_uses_parent_when_leaf_missing() {
        let base = std::env::temp_dir().join(format!("marius-picker-parent-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let missing_leaf = base.join("removed-subdir");
        let start = picker_start_dir(missing_leaf.to_string_lossy().as_ref());
        assert_eq!(start, base);
        let _ = fs::remove_dir_all(&base);
    }
}

pub fn resolve_output_dir(root: &Path, output_dir: &str) -> PathBuf {
    let trimmed = output_dir.trim();
    if trimmed.is_empty() {
        return root.join("output");
    }
    let p = PathBuf::from(trimmed);
    if p.is_absolute() {
        p
    } else {
        root.join(p)
    }
}
