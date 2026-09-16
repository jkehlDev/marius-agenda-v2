use super::navigation::WizardUi;
use super::session::{flush_config_to_workspace, set_project_output_dir};
use super::state::AppState;
use super::util;
use agenda_core::{OutputDirIssue, resolve_output_dir_for_generate, output_dir_issue_message};
use agenda_pipeline::{
    export_prepared_pdf_generation, prepare_pdf_generation, GenerateError, GenerateOptions,
    GenerateResult,
};
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

pub fn generate_period(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    period_id: &str,
    period_label: &str,
) {
    if state.borrow().generating {
        return;
    }

    super::state::apply_step_sync(state);
    if let Err(e) = flush_config_to_workspace(state) {
        util::message(&ui.window, "Une erreur est survenue", &e.to_string());
        return;
    }

    let root = state.borrow().root.clone();
    let out_cfg = state.borrow().config.output_dir.clone();
    match resolve_output_dir_for_generate(&root, &out_cfg) {
        Ok(_) => run_generation(ui, state, period_id, period_label),
        Err(OutputDirIssue::NotChosen) => {
            prompt_output_folder(ui, state, period_id, period_label, &out_cfg);
        }
        Err(OutputDirIssue::Missing(path)) => {
            let window = ui.window.clone();
            let ui = ui.clone_handles();
            let state = state.clone();
            let period_id = period_id.to_string();
            let period_label = period_label.to_string();
            let picker_start = out_cfg.clone();
            util::confirm_missing_output_dir(&window, &path, move || {
                prompt_output_folder(
                    &ui,
                    &state,
                    &period_id,
                    &period_label,
                    &picker_start,
                );
            });
        }
    }
}

fn prompt_output_folder(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    period_id: &str,
    period_label: &str,
    last_output_dir: &str,
) {
    let window = ui.window.clone();
    let ui = ui.clone_handles();
    let state = state.clone();
    let period_id = period_id.to_string();
    let period_label = period_label.to_string();
    let window_msg = window.clone();
    util::pick_output_folder(&window, last_output_dir, move |picked| {
        match picked {
            None => {}
            Some(picked) => {
                if let Err(e) = set_project_output_dir(&ui, &state, &picked) {
                    util::message(&window_msg, "Dossier de sortie", &e.to_string());
                    return;
                }
                super::refresh_step_page(&ui, &state, true);
                run_generation(&ui, &state, &period_id, &period_label);
            }
        }
    });
}

fn run_generation(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    period_id: &str,
    period_label: &str,
) {
    let window = ui.window.clone();

    let (root, options) = {
        let s = state.borrow();
        let workspace = s
            .workspace_dir()
            .cloned()
            .expect("project workspace for PDF");
        let dir = match resolve_output_dir_for_generate(&s.root, &s.config.output_dir) {
            Ok(d) => d,
            Err(issue) => {
                util::message(
                    &window,
                    "Dossier de sortie",
                    &output_dir_issue_message(&issue),
                );
                return;
            }
        };
        (
            s.root.clone(),
            GenerateOptions {
                period_ids: Some(vec![period_id.to_string()]),
                output_dir: Some(dir),
                data_dir: Some(workspace),
            },
        )
    };

    state.borrow_mut().generating = true;
    super::set_wizard_busy(ui, state, true);
    ui.gen_banner.set_revealed(true);

    let state = state.clone();
    let ui = ui.clone_handles();
    let period_label = period_label.to_string();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let prepared = prepare_pdf_generation(&root, options);
        let _ = tx.send(prepared);
    });

    glib::idle_add_local(move || {
        match rx.try_recv() {
            Ok(prepared) => {
                finish_generation_ui(&ui, &state, &period_label, prepared);
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                state.borrow_mut().generating = false;
                super::set_wizard_busy(&ui, &state, false);
                ui.gen_banner.set_revealed(false);
                util::message(
                    &ui.window,
                    "Une erreur est survenue",
                    "La préparation du PDF s’est arrêtée de façon inattendue.",
                );
                glib::ControlFlow::Break
            }
        }
    });
}

fn finish_generation_ui(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    period_label: &str,
    prepared: Result<agenda_pipeline::PreparedPdfGeneration, GenerateError>,
) {
    let window = &ui.window;
    let result: Result<GenerateResult, GenerateError> = match prepared {
        Ok(job) => export_prepared_pdf_generation(job),
        Err(e) => Err(e),
    };

    state.borrow_mut().generating = false;
    super::set_wizard_busy(ui, state, false);
    ui.gen_banner.set_revealed(false);

    match result {
        Ok(res) => {
            if let Some(name) = res.pdfs.first() {
                let pdf_path = res.pdf_dir.join(name);
                util::pdf_created_dialog(window, &pdf_path);
            } else {
                util::message(
                    window,
                    "PDF créé",
                    &format!(
                        "Génération terminée pour « {} », mais aucun fichier PDF n’a été produit.",
                        period_label
                    ),
                );
            }
        }
        Err(e) => util::message(window, "Une erreur est survenue", &e.to_string()),
    }
}
