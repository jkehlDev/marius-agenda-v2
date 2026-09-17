use super::navigation::WizardUi;
use super::state::AppState;
use agenda_core::{picker_start_dir, PROJECT_EXTENSION};
use gtk::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

pub fn message(window: &adw::ApplicationWindow, title: &str, body: &str) {
    let dlg = adw::MessageDialog::new(Some(window), Some(title), Some(body));
    dlg.add_response("ok", "Fermer");
    dlg.connect_response(None::<&str>, |dialog, _| dialog.close());
    dlg.present();
}

fn launch_default_uri(path: &Path) {
    let file = gtk::gio::File::for_path(path);
    let _ = gtk::gio::AppInfo::launch_default_for_uri(
        &file.uri(),
        None::<&gtk::gio::AppLaunchContext>,
    );
}

/// Success dialog after PDF export: full path on disk + open file / folder.
pub fn pdf_created_dialog(window: &adw::ApplicationWindow, pdf_path: &Path) {
    let path_display = pdf_path.display().to_string();
    let file_name = pdf_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path_display.clone());
    let parent_dir = pdf_path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let dlg = adw::MessageDialog::new(
        Some(window),
        Some("PDF créé"),
        Some("Le fichier a été enregistré ici :"),
    );
    dlg.add_css_class("marius-pdf-created-dialog");

    let details = gtk::Box::new(gtk::Orientation::Vertical, 6);
    details.set_size_request(480, -1);
    let file_lbl = gtk::Label::new(Some(&file_name));
    file_lbl.set_halign(gtk::Align::Start);
    file_lbl.set_xalign(0.0);
    file_lbl.set_wrap(false);
    file_lbl.set_selectable(true);
    file_lbl.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    file_lbl.add_css_class("marius-pdf-created-file");
    details.append(&file_lbl);

    if !parent_dir.is_empty() {
        let dir_lbl = gtk::Label::new(Some(&parent_dir));
        dir_lbl.set_halign(gtk::Align::Start);
        dir_lbl.set_xalign(0.0);
        dir_lbl.set_wrap(true);
        dir_lbl.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        dir_lbl.set_selectable(true);
        dir_lbl.add_css_class("dim-label");
        dir_lbl.add_css_class("marius-pdf-created-path");
        details.append(&dir_lbl);
    }

    dlg.set_extra_child(Some(&details));

    dlg.add_response("close", "Fermer");
    dlg.add_response("folder", "Ouvrir le dossier");
    dlg.add_response("open", "Ouvrir le PDF");
    dlg.set_response_appearance("open", adw::ResponseAppearance::Suggested);
    dlg.set_default_response(Some("open"));
    dlg.set_close_response("close");

    let pdf_path = pdf_path.to_path_buf();
    dlg.connect_response(None::<&str>, move |dialog, response| {
        match response {
            "open" => launch_default_uri(&pdf_path),
            "folder" => {
                if let Some(dir) = pdf_path.parent() {
                    launch_default_uri(dir);
                }
            }
            _ => {}
        }
        dialog.close();
    });
    dlg.present();
}

/// Option A: saved PDF folder gone — user chooses whether to pick a new one.
pub fn confirm_missing_output_dir(
    window: &adw::ApplicationWindow,
    saved_path: &Path,
    on_choose: impl FnOnce() + 'static,
) {
    let body = format!(
        "Le dossier enregistré pour les PDF n’existe plus :\n{}\n\nChoisir un nouveau dossier ?",
        saved_path.display()
    );
    let dlg = adw::MessageDialog::new(
        Some(window),
        Some("Dossier de sortie introuvable"),
        Some(&body),
    );
    dlg.add_response("cancel", "Annuler");
    dlg.add_response("choose", "Choisir…");
    dlg.set_response_appearance("choose", adw::ResponseAppearance::Suggested);
    dlg.set_default_response(Some("choose"));
    dlg.set_close_response("cancel");
    let on_choose = Mutex::new(Some(on_choose));
    dlg.connect_response(None::<&str>, move |dialog, response| {
        if response == "choose" {
            if let Some(f) = on_choose.lock().unwrap().take() {
                f();
            }
        }
        dialog.close();
    });
    dlg.present();
}

pub fn confirm_overwrite_pdfs(
    window: &adw::ApplicationWindow,
    existing_paths: &[PathBuf],
    on_confirm: impl FnOnce() + 'static,
) {
    let list = existing_paths
        .iter()
        .map(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.display().to_string())
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = if existing_paths.len() == 1 {
        format!(
            "Un fichier PDF existe déjà et sera remplacé :\n\n{}\n\nContinuer ?",
            list
        )
    } else {
        format!(
            "Ces fichiers PDF existent déjà et seront remplacés :\n\n{}\n\nContinuer ?",
            list
        )
    };
    let dlg = adw::MessageDialog::new(
        Some(window),
        Some("Remplacer le PDF existant ?"),
        Some(&body),
    );
    dlg.add_response("cancel", "Annuler");
    dlg.add_response("replace", "Remplacer");
    dlg.set_response_appearance("replace", adw::ResponseAppearance::Destructive);
    dlg.set_default_response(Some("cancel"));
    dlg.set_close_response("cancel");
    let on_confirm = Mutex::new(Some(on_confirm));
    dlg.connect_response(None::<&str>, move |dialog, response| {
        if response == "replace" {
            if let Some(f) = on_confirm.lock().unwrap().take() {
                f();
            }
        }
        dialog.close();
    });
    dlg.present();
}

pub fn confirm_destructive(
    window: &adw::ApplicationWindow,
    title: &str,
    body: &str,
    confirm_label: &str,
    on_confirm: impl FnOnce() + 'static,
) {
    let dlg = adw::MessageDialog::new(Some(window), Some(title), Some(body));
    dlg.add_response("cancel", "Annuler");
    dlg.add_response("confirm", confirm_label);
    dlg.set_response_appearance("confirm", adw::ResponseAppearance::Destructive);
    dlg.set_default_response(Some("cancel"));
    dlg.set_close_response("cancel");
    let on_confirm = Mutex::new(Some(on_confirm));
    dlg.connect_response(None::<&str>, move |dialog, response| {
        if response == "confirm" {
            if let Some(f) = on_confirm.lock().unwrap().take() {
                f();
            }
        }
        dialog.close();
    });
    dlg.present();
}

pub fn pick_image_file(window: &adw::ApplicationWindow, on_done: impl FnOnce(Option<PathBuf>) + 'static) {
    let dialog = gtk::FileDialog::builder()
        .title("Choisir une image")
        .modal(true)
        .build();
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Images"));
    filter.add_mime_type("image/*");
    dialog.set_default_filter(Some(&filter));
    dialog.open(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| on_done(result.ok().and_then(|f| f.path())),
    );
}

pub enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}

pub fn confirm_unsaved_changes(
    window: &adw::ApplicationWindow,
    on_choice: impl FnOnce(UnsavedChoice) + 'static,
) {
    let dlg = adw::MessageDialog::new(
        Some(window),
        Some("Modifications non enregistrées"),
        Some("Enregistrer le projet avant de continuer ?"),
    );
    dlg.add_response("cancel", "Annuler");
    dlg.add_response("discard", "Ne pas enregistrer");
    dlg.add_response("save", "Enregistrer");
    dlg.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dlg.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dlg.set_default_response(Some("save"));
    dlg.set_close_response("cancel");
    let on_choice = Mutex::new(Some(on_choice));
    dlg.connect_response(None::<&str>, move |dialog, response| {
        let choice = match response {
            "save" => UnsavedChoice::Save,
            "discard" => UnsavedChoice::Discard,
            _ => UnsavedChoice::Cancel,
        };
        if let Some(f) = on_choice.lock().unwrap().take() {
            f(choice);
        }
        dialog.close();
    });
    dlg.present();
}

fn marius_file_filter() -> gtk::FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Projet Marius"));
    filter.add_pattern(&format!("*.{}", PROJECT_EXTENSION));
    filter.add_mime_type("application/zip");
    filter
}

pub fn pick_open_marius_file(
    window: &adw::ApplicationWindow,
    on_done: impl FnOnce(Option<PathBuf>) + 'static,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Ouvrir un projet Marius")
        .modal(true)
        .build();
    dialog.set_default_filter(Some(&marius_file_filter()));
    dialog.open(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| on_done(result.ok().and_then(|f| f.path())),
    );
}

pub fn pick_save_marius_file(
    window: &adw::ApplicationWindow,
    default_name: &str,
    on_done: impl FnOnce(Option<PathBuf>) + 'static,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Enregistrer le projet")
        .modal(true)
        .initial_name(&format!("{}.{}", default_name, PROJECT_EXTENSION))
        .build();
    dialog.set_default_filter(Some(&marius_file_filter()));
    dialog.save(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| on_done(result.ok().and_then(|f| f.path())),
    );
}

pub fn pick_output_folder(
    window: &adw::ApplicationWindow,
    last_output_dir: &str,
    on_done: impl FnOnce(Option<PathBuf>) + 'static,
) {
    let dialog = gtk::FileDialog::builder()
        .title("Enregistrer les PDF dans…")
        .modal(true)
        .build();
    let start = picker_start_dir(last_output_dir);
    dialog.set_initial_folder(Some(&gtk::gio::File::for_path(&start)));
    dialog.select_folder(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| on_done(result.ok().and_then(|f| f.path())),
    );
}

/// GTK `Fn` handler: clone handles, run `action`, rebuild current step body.
pub fn on_click_refresh(
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
    action: impl Fn(&Rc<RefCell<AppState>>) + 'static,
) -> impl Fn(&gtk::Button) + 'static {
    let state = state.clone();
    let ui = ui.clone_handles();
    move |_| {
        action(&state);
        super::refresh_step_page(&ui, &state, true);
    }
}
