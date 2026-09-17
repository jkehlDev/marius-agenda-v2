//! Header main menu, About, keyboard shortcuts (GNOME HIG).

use super::icons;
use gtk::gio;
use gtk::prelude::*;
use libadwaita as adw;
use std::path::Path;

pub fn main_menu_button(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    app_root: &Path,
) -> gtk::MenuButton {
    wire_about_action(app, window, app_root);
    wire_show_shortcuts_action(window);

    let menu = gio::Menu::new();
    menu.append(Some("Raccourcis clavier"), Some("win.show-shortcuts"));
    menu.append(Some("À propos de Marius Agenda"), Some("app.about"));

    let btn = gtk::MenuButton::new();
    btn.set_icon_name("open-menu-symbolic");
    btn.set_tooltip_text(Some("Menu principal"));
    btn.set_menu_model(Some(&menu));
    btn.set_primary(true);
    btn
}

fn wire_about_action(app: &adw::Application, window: &adw::ApplicationWindow, app_root: &Path) {
    let action = gio::SimpleAction::new("about", None);
    let window = window.clone();
    let app_root = app_root.to_path_buf();
    action.connect_activate(move |_, _| present_about(&window, &app_root));
    app.add_action(&action);
}

fn wire_show_shortcuts_action(window: &adw::ApplicationWindow) {
    let action = gio::SimpleAction::new("show-shortcuts", None);
    let parent = window.clone();
    action.connect_activate(move |_, _| {
        let shortcuts = build_shortcuts_window(&parent);
        shortcuts.present();
    });
    window.add_action(&action);
}

fn build_shortcuts_window(parent: &adw::ApplicationWindow) -> gtk::ShortcutsWindow {
    let win = gtk::ShortcutsWindow::builder()
        .transient_for(parent)
        .modal(true)
        .title("Raccourcis clavier — Marius Agenda")
        .build();

    if let Some(app) = parent.application() {
        app.add_window(&win);
    }

    let section = gtk::ShortcutsSection::builder().title("Projet").build();
    let save = gtk::ShortcutsShortcut::builder()
        .title("Enregistrer le projet")
        .action_name("win.save")
        .accelerator("<Primary>s")
        .build();
    section.append(&save);
    win.add_section(&section);
    win
}

pub fn present_about(parent: &adw::ApplicationWindow, app_root: &Path) {
    icons::register_app_icons(app_root);
    let about = adw::AboutWindow::new();
    about.set_transient_for(Some(parent));
    about.set_modal(true);
    about.set_application_name("Marius Agenda");
    about.set_application_icon(icons::icon_name_for_about());
    about.set_version(env!("CARGO_PKG_VERSION"));
    about.set_copyright("Copyright © 2026 jkehlDev");
    about.set_license_type(gtk::License::Gpl30);
    about.set_developers(&["jkehlDev"]);
    about.set_website("https://github.com/jkehlDev/marius-agenda-v2");
    about.set_issue_url("https://github.com/jkehlDev/marius-agenda-v2/issues");
    about.set_comments(
        "Générateur d’agenda scolaire en PDF (page à page ou livret imposé).",
    );
    about.present();
}
