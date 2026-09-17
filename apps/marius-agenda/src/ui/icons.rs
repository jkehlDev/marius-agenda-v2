//! App icon: dev tree (`packaging/icons/hicolor`) and installed hicolor theme.

use gtk::gdk;
use gtk::prelude::*;
use std::path::Path;

const ICON_NAME: &str = "marius-agenda";

pub fn register_app_icons(app_root: &Path) {
    let hicolor = app_root.join("packaging/icons/hicolor");
    if !hicolor.is_dir() {
        return;
    }
    let display = gdk::Display::default().expect("display");
    gtk::IconTheme::for_display(&display).add_search_path(&hicolor);
}

pub fn apply_window_icon(window: &impl IsA<gtk::Window>, app_root: &Path) {
    register_app_icons(app_root);
    window.set_icon_name(Some(ICON_NAME));
}

pub fn apply_status_page_icon(page: &libadwaita::StatusPage, app_root: &Path) {
    register_app_icons(app_root);
    if gtk::IconTheme::for_display(&gdk::Display::default().expect("display"))
        .has_icon(ICON_NAME)
    {
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

fn load_icon_texture(app_root: &Path) -> Option<gdk::Texture> {
    for name in ["marius-agenda-256.png", "marius-agenda-128.png", "marius-agenda-48.png"] {
        let path = app_root.join("packaging/icons").join(name);
        if path.is_file() {
            return gdk::Texture::from_filename(&path).ok();
        }
    }
    None
}
