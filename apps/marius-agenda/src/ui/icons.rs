//! App icon: dev tree (`packaging/icons/hicolor`) and installed hicolor theme.

use gtk::gdk;
use gtk::prelude::*;
use std::path::{Path, PathBuf};

const ICON_NAME: &str = "marius-agenda";

/// GTK icon theme search path: directory that **contains** a `hicolor/` subfolder (not `hicolor` itself).
pub fn register_app_icons(app_root: &Path) {
    let theme = gtk::IconTheme::for_display(&gdk::Display::default().expect("display"));
    for path in icon_search_paths(app_root) {
        if path.join("hicolor").is_dir() {
            let canonical = path.canonicalize().unwrap_or(path);
            theme.add_search_path(&canonical);
        }
    }
}

pub fn apply_window_icon(window: &impl IsA<gtk::Window>, app_root: &Path) {
    register_app_icons(app_root);
    window.set_icon_name(Some(ICON_NAME));
}

pub fn apply_status_page_icon(page: &libadwaita::StatusPage, app_root: &Path) {
    register_app_icons(app_root);
    if icon_available() {
        page.set_icon_name(Some(ICON_NAME));
        page.set_paintable(None::<&gdk::Paintable>);
        return;
    }
    if let Some(texture) = load_icon_texture(app_root) {
        page.set_icon_name(None);
        page.set_paintable(Some(&texture));
    } else {
        page.set_icon_name(Some("calendar-month-symbolic"));
    }
}

pub fn icon_name_for_about() -> &'static str {
    ICON_NAME
}

fn icon_available() -> bool {
    gtk::IconTheme::for_display(&gdk::Display::default().expect("display")).has_icon(ICON_NAME)
}

fn icon_search_paths(app_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let bundled = app_root.join("packaging/icons");
    if bundled.join("hicolor").is_dir() {
        paths.push(bundled);
    }
    if let Some(user) = install_bundled_icons_to_xdg(app_root) {
        paths.push(user);
    }
    paths
}

/// Copy PNGs into `$XDG_DATA_HOME/marius-agenda/icons/hicolor` when theme lookup fails (e.g. symlinks).
fn install_bundled_icons_to_xdg(app_root: &Path) -> Option<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share"))
        })?;
    let icons_root = data_home.join("marius-agenda").join("icons");
    let sizes = [
        ("48x48", "marius-agenda-48.png"),
        ("128x128", "marius-agenda-128.png"),
        ("256x256", "marius-agenda-256.png"),
    ];
    let mut wrote = false;
    for (size, src_name) in sizes {
        let src = app_root.join("packaging/icons").join(src_name);
        if !src.is_file() {
            continue;
        }
        let dest = icons_root
            .join("hicolor")
            .join(size)
            .join("apps")
            .join("marius-agenda.png");
        if dest.is_file() {
            wrote = true;
            continue;
        }
        if let Some(parent) = dest.parent() {
            if std::fs::create_dir_all(parent).is_ok() && std::fs::copy(&src, &dest).is_ok() {
                wrote = true;
            }
        }
    }
    if wrote || icons_root.join("hicolor").is_dir() {
        Some(icons_root)
    } else {
        None
    }
}

fn load_icon_texture(app_root: &Path) -> Option<gdk::Texture> {
    for name in ["marius-agenda-256.png", "marius-agenda-128.png", "marius-agenda-48.png"] {
        let path = app_root.join("packaging/icons").join(name);
        if path.is_file() {
            return gdk::Texture::from_filename(&path).ok();
        }
    }
    None
}
