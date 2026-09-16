//! Project session (.marius): dirty flag, save/open, navigation home ↔ wizard.

use super::navigation::WizardUi;
use super::state::{apply_step_sync, AppState, AppScreen};
use super::refresh_step_page;
use agenda_core::{
    create_new_project, note_recent_open, open_project_archive, remove_ephemeral_workspace,
    save_project_archive, write_workspace_config, OpenProject, ProjectError,
};
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub fn project_display_title(state: &AppState) -> String {
    let name = state
        .project
        .as_ref()
        .map(|p| p.display_name.as_str())
        .unwrap_or("Sans titre");
    if state.project.as_ref().is_some_and(|p| p.dirty) {
        format!("{} *", name)
    } else {
        name.to_string()
    }
}

pub fn update_window_title(ui: &WizardUi, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    if s.screen != AppScreen::Wizard {
        ui.window.set_title(Some("Marius Agenda"));
        return;
    }
    let title = format!("Marius Agenda — {}", project_display_title(&s));
    ui.window.set_title(Some(&title));
}

/// Stores PDF output folder in project config (+ workspace; auto-saves `.marius` when path known).
pub fn set_project_output_dir(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    path: &Path,
) -> Result<(), ProjectError> {
    let stored = path
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());
    {
        let mut s = state.borrow_mut();
        s.config.output_dir = stored;
        s.mark_project_dirty();
    }
    flush_config_to_workspace(state)?;
    let archive = state
        .borrow()
        .project
        .as_ref()
        .and_then(|p| p.archive_path.clone());
    if let Some(archive) = archive {
        persist_archive(state, &archive)?;
        update_window_title(ui, state);
    }
    Ok(())
}

pub fn flush_config_to_workspace(state: &Rc<RefCell<AppState>>) -> Result<(), ProjectError> {
    apply_step_sync(state);
    let s = state.borrow();
    let proj = s.project.as_ref().ok_or(ProjectError::NotSaved)?;
    write_workspace_config(&proj.workspace_dir, &s.config)
}

fn release_active_workspace(state: &Rc<RefCell<AppState>>) {
    let workspace = state
        .borrow()
        .project
        .as_ref()
        .map(|p| p.workspace_dir.clone());
    if let Some(dir) = workspace {
        remove_ephemeral_workspace(&dir);
    }
}

pub fn start_new_project(ui: &WizardUi, state: &Rc<RefCell<AppState>>) -> Result<(), ProjectError> {
    release_active_workspace(state);
    let created = create_new_project()?;
    {
        let mut s = state.borrow_mut();
        s.config = created.config;
        s.step = 0;
        s.project = Some(super::state::ActiveProject {
            workspace_dir: created.workspace_dir,
            archive_path: None,
            display_name: "Sans titre".into(),
            dirty: false,
        });
        s.screen = AppScreen::Wizard;
    }
    ui.app_stack.set_visible_child_name("wizard");
    sync_header_chrome(ui, state);
    refresh_step_page(ui, state, true);
    update_window_title(ui, state);
    Ok(())
}

pub fn open_project(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    path: PathBuf,
) -> Result<(), ProjectError> {
    release_active_workspace(state);
    let opened: OpenProject = open_project_archive(&path)?;
    note_recent_open(&opened.archive_path, &opened.display_name);
    {
        let mut s = state.borrow_mut();
        s.config = opened.config;
        s.step = opened.wizard_step;
        s.project = Some(super::state::ActiveProject {
            workspace_dir: opened.workspace_dir,
            archive_path: Some(opened.archive_path),
            display_name: opened.display_name,
            dirty: false,
        });
        s.screen = AppScreen::Wizard;
    }
    ui.app_stack.set_visible_child_name("wizard");
    sync_header_chrome(ui, state);
    refresh_step_page(ui, state, true);
    update_window_title(ui, state);
    Ok(())
}

pub fn enter_home(ui: &WizardUi, state: &Rc<RefCell<AppState>>) {
    release_active_workspace(state);
    state.borrow_mut().screen = AppScreen::Home;
    state.borrow_mut().project = None;
    ui.app_stack.set_visible_child_name("home");
    ui.window.set_title(Some("Marius Agenda"));
    sync_header_chrome(ui, state);
    super::home::refresh_recent_list(&ui.home_recent_list, state, ui);
}

pub fn sync_header_chrome(ui: &WizardUi, state: &Rc<RefCell<AppState>>) {
    let on_wizard = state.borrow().in_wizard();
    ui.home_btn.set_sensitive(on_wizard);
    ui.save_menu_btn.set_sensitive(on_wizard);
}

pub fn persist_archive(state: &Rc<RefCell<AppState>>, archive_path: &Path) -> Result<(), ProjectError> {
    flush_config_to_workspace(state)?;
    let (workspace, title, wizard_step) = {
        let s = state.borrow();
        let p = s.project.as_ref().ok_or(ProjectError::NotSaved)?;
        let title = p.display_name.clone();
        (p.workspace_dir.clone(), title, s.step)
    };
    let config = state.borrow().config.clone();
    save_project_archive(&workspace, archive_path, &config, &title, wizard_step)?;
    let mut s = state.borrow_mut();
    if let Some(p) = s.project.as_mut() {
        p.archive_path = Some(archive_path.to_path_buf());
        if let Some(stem) = archive_path.file_stem().and_then(|n| n.to_str()) {
            p.display_name = stem.to_string();
        }
        p.dirty = false;
    }
    Ok(())
}

pub fn save_project_known_path(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
) -> Result<(), ProjectError> {
    let path = state
        .borrow()
        .project
        .as_ref()
        .and_then(|p| p.archive_path.clone())
        .ok_or(ProjectError::NotSaved)?;
    persist_archive(state, &path)?;
    update_window_title(ui, state);
    Ok(())
}

pub fn run_with_unsaved_guard(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    on_proceed: impl FnOnce(&WizardUi, &Rc<RefCell<AppState>>) + 'static,
) {
    if state.borrow().screen != AppScreen::Wizard {
        on_proceed(ui, state);
        return;
    }
    if !state.borrow().project.as_ref().is_some_and(|p| p.dirty) {
        on_proceed(ui, state);
        return;
    }
    let ui_c = ui.clone_handles();
    let state_c = state.clone();
    super::util::confirm_unsaved_changes(&ui.window, move |choice| match choice {
        super::util::UnsavedChoice::Cancel => {}
        super::util::UnsavedChoice::Discard => on_proceed(&ui_c, &state_c),
        super::util::UnsavedChoice::Save => {
            let ui_s = ui_c.clone_handles();
            let state_s = state_c.clone();
            let ui_done = ui_s.clone_handles();
            let state_done = state_s.clone();
            save_or_prompt_as(&ui_s, &state_s, move || on_proceed(&ui_done, &state_done));
        }
    });
}

pub fn save_project_as(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    on_success: impl FnOnce() + 'static,
) {
    let name = state
        .borrow()
        .project
        .as_ref()
        .map(|p| {
            p.archive_path
                .as_ref()
                .and_then(|a| a.file_stem())
                .and_then(|s| s.to_str())
                .unwrap_or(p.display_name.as_str())
                .to_string()
        })
        .unwrap_or_else(|| "Sans titre".to_string());
    let window = ui.window.clone();
    let ui = ui.clone_handles();
    let state = state.clone();
    super::util::pick_save_marius_file(&window, &name, move |picked| {
        if let Some(path) = picked {
            match persist_archive(&state, &path) {
                Ok(()) => {
                    update_window_title(&ui, &state);
                    on_success();
                }
                Err(e) => super::util::message(&ui.window, "Enregistrement", &e.to_string()),
            }
        }
    });
}

pub fn save_or_prompt_as(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    on_success: impl FnOnce() + 'static,
) {
    if state.borrow().project.as_ref().and_then(|p| p.archive_path.clone()).is_some() {
        match save_project_known_path(ui, state) {
            Ok(()) => on_success(),
            Err(e) => super::util::message(&ui.window, "Enregistrement", &e.to_string()),
        }
        return;
    }
    let name = state
        .borrow()
        .project
        .as_ref()
        .map(|p| p.display_name.clone())
        .unwrap_or_else(|| "Sans titre".to_string());
    let window = ui.window.clone();
    let ui = ui.clone_handles();
    let state = state.clone();
    super::util::pick_save_marius_file(&window, &name, move |picked| {
        if let Some(path) = picked {
            match persist_archive(&state, &path) {
                Ok(()) => {
                    update_window_title(&ui, &state);
                    on_success();
                }
                Err(e) => super::util::message(&ui.window, "Enregistrement", &e.to_string()),
            }
        }
    });
}

/// GTK `close-request` handler: allow close on home or clean wizard; prompt if dirty.
pub fn handle_window_close_request(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
) -> gtk::glib::Propagation {
    if state.borrow().screen == AppScreen::Home {
        return gtk::glib::Propagation::Proceed;
    }
    if !state.borrow().project.as_ref().is_some_and(|p| p.dirty) {
        return gtk::glib::Propagation::Proceed;
    }
    run_with_unsaved_guard(ui, state, |ui, state| {
        if let Some(p) = state.borrow_mut().project.as_mut() {
            p.dirty = false;
        }
        ui.window.close();
    });
    gtk::glib::Propagation::Stop
}
