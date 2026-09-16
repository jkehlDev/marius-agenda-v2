//! Application home — new / open `.marius` project.

use super::layout::trash_icon_button;
use super::navigation::WizardUi;
use super::session::{open_project, start_new_project};
use super::state::AppState;
use super::util::{confirm_destructive, message};
use agenda_core::{delete_project_archive, load_recent_projects};
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Label, Orientation, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

pub fn refresh_recent_list(
    recent_box: &GtkBox,
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
) {
    while let Some(child) = recent_box.first_child() {
        recent_box.remove(&child);
    }

    let recents = load_recent_projects();
    if recents.entries.is_empty() {
        let empty = Label::new(Some("Aucun projet récent pour l’instant."));
        empty.add_css_class("dim-label");
        empty.set_halign(Align::Start);
        recent_box.append(&empty);
        return;
    }

    for entry in recents.entries {
        let row_line = GtkBox::new(Orientation::Horizontal, 6);
        row_line.add_css_class("marius-recent-row");
        row_line.set_halign(Align::Fill);

        let del_btn = trash_icon_button("Supprimer le fichier de projet…");
        del_btn.set_valign(Align::Center);
        del_btn.set_halign(Align::End);

        let open_btn = gtk::Button::new();
        open_btn.add_css_class("flat");
        open_btn.set_hexpand(true);
        open_btn.set_halign(Align::Fill);
        let inner = GtkBox::new(Orientation::Vertical, 2);
        let name = Label::new(Some(&entry.title));
        name.set_halign(Align::Start);
        name.add_css_class("heading");
        let path_lbl = Label::new(Some(&entry.path.display().to_string()));
        path_lbl.set_halign(Align::Start);
        path_lbl.add_css_class("dim-label");
        path_lbl.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        inner.append(&name);
        inner.append(&path_lbl);
        open_btn.set_child(Some(&inner));

        let path = entry.path.clone();
        let title = entry.title.clone();
        let ui_r = ui.clone_handles();
        let state_r = state.clone();
        open_btn.connect_clicked(move |_| {
            if let Err(e) = open_project(&ui_r, &state_r, path.clone()) {
                message(&ui_r.window, "Ouverture", &e.to_string());
            }
        });

        let path_del = entry.path.clone();
        let recent_box_del = recent_box.clone();
        let ui_del = ui.clone_handles();
        let state_del = state.clone();
        let window = ui.window.clone();
        del_btn.connect_clicked(move |_| {
            let body = format!(
                "Le fichier sera supprimé définitivement du disque.\n\n{}\n{}",
                title,
                path_del.display()
            );
            let path_confirm = path_del.clone();
            let recent_box_confirm = recent_box_del.clone();
            let ui_confirm = ui_del.clone_handles();
            let state_confirm = state_del.clone();
            confirm_destructive(
                &window,
                "Supprimer le projet",
                &body,
                "Supprimer",
                move || {
                    if let Err(e) = delete_project_archive(&path_confirm) {
                        message(&ui_confirm.window, "Suppression", &e.to_string());
                    }
                    refresh_recent_list(&recent_box_confirm, &state_confirm, &ui_confirm);
                },
            );
        });

        row_line.append(&open_btn);
        row_line.append(&del_btn);
        recent_box.append(&row_line);
    }
}

pub fn build_home_page(
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
    recent_box: &GtkBox,
) -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 20);
    page.set_margin_top(32);
    page.set_margin_bottom(32);
    page.set_margin_start(24);
    page.set_margin_end(24);
    page.add_css_class("marius-home");

    let title = Label::new(Some("Marius Agenda"));
    title.add_css_class("title-1");
    title.set_halign(Align::Start);
    page.append(&title);

    let lede = Label::new(Some(
        "Crée ou ouvre un projet d’agenda (.marius). Ton travail est enregistré dans un fichier unique à partager ou archiver.",
    ));
    lede.set_wrap(true);
    lede.add_css_class("dim-label");
    lede.set_halign(Align::Start);
    page.append(&lede);

    let actions = GtkBox::new(Orientation::Horizontal, 12);
    actions.set_margin_top(8);
    let new_btn = gtk::Button::with_label("Nouveau projet");
    new_btn.add_css_class("suggested-action");
    let open_btn = gtk::Button::with_label("Ouvrir un projet…");
    actions.append(&new_btn);
    actions.append(&open_btn);
    page.append(&actions);

    let recent_title = Label::new(Some("Projets récents"));
    recent_title.add_css_class("heading");
    recent_title.set_halign(Align::Start);
    recent_title.set_margin_top(16);
    page.append(&recent_title);

    refresh_recent_list(recent_box, state, ui);

    let scroll = ScrolledWindow::builder().vexpand(true).build();
    scroll.set_child(Some(recent_box));
    page.append(&scroll);

    let ui_new = ui.clone_handles();
    let state_new = state.clone();
    new_btn.connect_clicked(move |_| {
        if let Err(e) = start_new_project(&ui_new, &state_new) {
            super::util::message(&ui_new.window, "Nouveau projet", &e.to_string());
        }
    });

    let ui_open = ui.clone_handles();
    let state_open = state.clone();
    open_btn.connect_clicked(move |_| {
        let window = ui_open.window.clone();
        let ui_open = ui_open.clone_handles();
        let state_open = state_open.clone();
        super::util::pick_open_marius_file(&window, move |picked| {
            if let Some(path) = picked {
                if let Err(e) = open_project(&ui_open, &state_open, path) {
                    super::util::message(&ui_open.window, "Ouverture", &e.to_string());
                }
            }
        });
    });

    page
}
