//! GTK home + 5-step wizard.

mod date_field;
mod generate;
mod home;
mod layout;
mod navigation;
mod screenshot;
mod session;
mod state;
mod steps;
mod util;

use agenda_core::{
    format_wizard_errors, load_recent_projects, prune_stale_workspace_dirs,
    remove_ephemeral_workspace, validate_wizard_step, WIZARD_STEP_COUNT,
};
use gtk::gdk::Key;
use gtk::gio::{ApplicationFlags, Cancellable};
use gtk::glib;
use gtk::prelude::*;
use layout::{load_app_css, CONTENT_MAX_WIDTH};
use libadwaita as adw;
use libadwaita::prelude::*;
use navigation::{perform_wizard_goto, perform_wizard_next, perform_wizard_prev, WizardUi};
use state::AppState;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

const STEPS: [&str; 5] = [
    "Année scolaire",
    "Vacances",
    "Jours de la semaine",
    "Illustrations",
    "Générer",
];

fn step_short_label(i: usize) -> &'static str {
    match i {
        0 => "Année",
        1 => "Vacances",
        2 => "Jours",
        3 => "Illustrations",
        _ => "Générer",
    }
}

pub fn run(root: PathBuf, open_project: Option<PathBuf>) {
    let _ = run_application(root, RunMode::Interactive(open_project));
}

pub fn run_gui_self_test(root: PathBuf) -> Result<(), String> {
    run_application(root, RunMode::SelfTest)
}

pub fn run_gui_screenshot_tour(root: PathBuf, out_dir: PathBuf) -> Result<(), String> {
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    run_application(root, RunMode::ScreenshotTour(out_dir))
}

enum RunMode {
    Interactive(Option<PathBuf>),
    SelfTest,
    ScreenshotTour(PathBuf),
}

fn run_application(root: PathBuf, mode: RunMode) -> Result<(), String> {
    agenda_pipeline::prepare_gtk_runtime();
    prune_stale_workspace_dirs();
    let _ = load_recent_projects();
    load_app_css();

    let result = Rc::new(RefCell::new(None::<Result<(), String>>));
    let result_cb = result.clone();

    let app_id = if matches!(mode, RunMode::SelfTest) {
        // Avoid D-Bus single-instance clash with installed snap / dev GUI (CI headless).
        "com.jkehldev.mariusagenda.selftest"
    } else {
        "com.jkehldev.mariusagenda"
    };
    let app = adw::Application::builder().application_id(app_id).build();

    let state = Rc::new(RefCell::new(AppState::new(root)));

    let (self_test, screenshot_dir, open_on_start) = match mode {
        RunMode::SelfTest => (true, None, None),
        RunMode::ScreenshotTour(d) => (false, Some(d), None),
        RunMode::Interactive(p) => (false, None, p),
    };
    app.connect_activate(move |app| {
        let ui = build_ui(app, state.clone());
        if self_test {
                let state_idle = state.clone();
                let result_idle = result_cb.clone();
                let ui_idle = ui.clone_handles();
                glib::idle_add_local_once(move || {
                    if let Err(e) = session::start_new_project(&ui_idle, &state_idle) {
                        *result_idle.borrow_mut() = Some(Err(e.to_string()));
                    } else {
                        *result_idle.borrow_mut() =
                            Some(execute_gui_self_test(&ui_idle, &state_idle));
                    }
                    ui_idle.window.application().expect("app").quit();
                });
        } else if let Some(out_dir) = screenshot_dir.clone() {
                let state_idle = state.clone();
                let ui_idle = ui.clone_handles();
                glib::idle_add_local_once(move || {
                    start_screenshot_tour(&ui_idle, &state_idle, out_dir, TourShot::Home);
                });
        } else if let Some(path) = open_on_start.clone() {
            let ui_open = ui.clone_handles();
            let state_open = state.clone();
            glib::idle_add_local_once(move || {
                if let Err(e) = session::open_project(&ui_open, &state_open, path) {
                    util::message(&ui_open.window, "Ouverture", &e.to_string());
                }
            });
        }
    });

    if self_test {
        app.set_flags(ApplicationFlags::NON_UNIQUE);
        app.register(Cancellable::NONE)
            .map_err(|e| format!("gui self-test register: {e}"))?;
        app.activate();
    }

    app.run_with_args(&[] as &[&str]);

    if self_test {
        result
            .borrow()
            .clone()
            .unwrap_or_else(|| Err("GUI self-test did not run (no activate)".into()))
    } else {
        Ok(())
    }
}

enum TourShot {
    Home,
    Wizard(usize),
}

/// Screenshot tour only — bypasses pill `can_goto_step` (back-only) rule.
fn force_tour_step(ui: &WizardUi, state: &Rc<RefCell<AppState>>, target: usize) {
    state.borrow_mut().step = target.min(WIZARD_STEP_COUNT - 1);
    refresh_step_page(ui, state, true);
}

fn start_screenshot_tour(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    out_dir: PathBuf,
    shot: TourShot,
) {
    let ui = ui.clone_handles();
    let state = state.clone();
    glib::timeout_add_local_once(Duration::from_millis(400), move || {
        match shot {
            TourShot::Home => {
                ui.window.set_title(Some("Marius Agenda"));
                eprintln!("screenshot tour: home");
                screenshot::capture_widget_png(&ui.window, &out_dir.join("step-00-home.png"));
                if let Err(e) = session::start_new_project(&ui, &state) {
                    eprintln!("screenshot tour: new project failed: {}", e);
                    ui.window.application().expect("app").quit();
                } else {
                    start_screenshot_tour(&ui, &state, out_dir, TourShot::Wizard(0));
                }
            }
            TourShot::Wizard(step) => {
                force_tour_step(&ui, &state, step);
                session::update_window_title(&ui, &state);
                eprintln!("screenshot tour: wizard step {}", step);
                let path = out_dir.join(format!("step-{:02}.png", step + 1));
                screenshot::capture_widget_png(&ui.window, &path);
                let next = step + 1;
                if next >= WIZARD_STEP_COUNT {
                    ui.window.application().expect("app").quit();
                } else {
                    start_screenshot_tour(&ui, &state, out_dir, TourShot::Wizard(next));
                }
            }
        }
    });
}

fn execute_gui_self_test(ui: &WizardUi, state: &Rc<RefCell<AppState>>) -> Result<(), String> {
    use crate::wizard_nav::NextClickOutcome;

    if state.borrow().step != 0 {
        return Err(format!("expected step 0, got {}", state.borrow().step));
    }

    for target in 1..WIZARD_STEP_COUNT {
        let outcome = perform_wizard_next(ui, state);
        if outcome != NextClickOutcome::Advanced(target) {
            return Err(format!(
                "step {}: expected Advanced({}), got {:?}",
                target - 1,
                target,
                outcome
            ));
        }
    }

    perform_wizard_prev(ui, state);
    if state.borrow().step != WIZARD_STEP_COUNT - 2 {
        return Err(format!(
            "after prev expected step {}, got {}",
            WIZARD_STEP_COUNT - 2,
            state.borrow().step
        ));
    }

    if perform_wizard_next(ui, state) != NextClickOutcome::Advanced(WIZARD_STEP_COUNT - 1) {
        return Err("expected Advanced(4) after prev+next".into());
    }

    perform_wizard_goto(ui, state, 0);
    if state.borrow().step != 0 {
        return Err(format!("pill goto 0 failed, step {}", state.borrow().step));
    }

    // Regression: output_dir edit + refresh_step_page must not nest RefCell borrows (PDF folder pick).
    force_tour_step(ui, state, WIZARD_STEP_COUNT - 1);
    state::edit_state_then(
        state,
        |s| {
            s.config.output_dir = "/tmp/marius-gui-self-test-pdf-out".into();
            s.mark_project_dirty();
        },
        || refresh_step_page(ui, state, true),
    );
    if state.borrow().config.output_dir != "/tmp/marius-gui-self-test-pdf-out" {
        return Err("generate step: output_dir not persisted after refresh".into());
    }
    if state.borrow().step != WIZARD_STEP_COUNT - 1 {
        return Err(format!(
            "generate step refresh changed step to {}",
            state.borrow().step
        ));
    }

    let gen_page = ui
        .stack
        .child_by_name(&(WIZARD_STEP_COUNT - 1).to_string())
        .expect("generate page");
    let gen_box = gen_page.downcast_ref::<gtk::Box>().expect("generate box");
    if gen_box.first_child().is_none() {
        return Err("generate step body is empty after refresh".into());
    }

    Ok(())
}

fn build_ui(app: &adw::Application, state: Rc<RefCell<AppState>>) -> WizardUi {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Marius Agenda")
        .default_width(920)
        .default_height(880)
        .build();
    window.set_size_request(760, 640);

    let header = adw::HeaderBar::new();

    let home_btn = gtk::Button::from_icon_name("go-home-symbolic");
    home_btn.set_tooltip_text(Some("Retour à l’accueil"));
    header.pack_start(&home_btn);

    let save_menu = gtk::gio::Menu::new();
    save_menu.append(Some("Enregistrer"), Some("win.save"));
    save_menu.append(Some("Enregistrer sous…"), Some("win.save-as"));
    let save_menu_btn = gtk::MenuButton::new();
    save_menu_btn.set_icon_name("document-save-symbolic");
    save_menu_btn.set_tooltip_text(Some("Enregistrer le projet"));
    save_menu_btn.set_menu_model(Some(&save_menu));
    header.pack_end(&save_menu_btn);

    let app_stack = gtk::Stack::new();
    app_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    app_stack.set_vexpand(true);

    let hero = gtk::Box::new(gtk::Orientation::Vertical, 10);
    hero.set_margin_top(16);
    let eyebrow = gtk::Label::new(Some("Générateur d'agenda scolaire"));
    eyebrow.add_css_class("dim-label");
    eyebrow.set_halign(gtk::Align::Start);
    let h1 = gtk::Label::new(Some("Marius Agenda"));
    h1.add_css_class("title-1");
    h1.set_halign(gtk::Align::Start);
    let lede = gtk::Label::new(Some(
        "Suis les étapes : calendrier, vacances, jours, illustrations, puis génère un PDF page à page ou un livret prêt à imprimer.",
    ));
    lede.set_wrap(true);
    lede.add_css_class("dim-label");
    lede.add_css_class("marius-hero-lede");
    lede.set_halign(gtk::Align::Start);
    hero.append(&eyebrow);
    hero.append(&h1);
    hero.append(&lede);

    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_vexpand(false);
    stack.set_valign(gtk::Align::Start);

    let step_title = gtk::Label::new(None);
    step_title.add_css_class("title-2");
    step_title.add_css_class("marius-step-title");
    step_title.set_halign(gtk::Align::Start);
    step_title.set_margin_end(12);

    let step_body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    step_body.add_css_class("marius-scroll-content");
    step_body.set_margin_end(6);
    step_body.append(&step_title);
    step_body.append(&stack);

    let scrolled = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .propagate_natural_height(true)
        .build();
    scrolled.add_css_class("marius-scrolled");
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_overlay_scrolling(false);
    scrolled.set_child(Some(&step_body));

    for i in 0..WIZARD_STEP_COUNT {
        let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
        page.set_margin_top(4);
        page.set_margin_bottom(12);
        page.set_margin_start(2);
        page.set_margin_end(16);
        page.set_vexpand(false);
        page.set_valign(gtk::Align::Start);
        stack.add_named(&page, Some(&i.to_string()));
    }

    let pills_row = gtk::FlowBox::new();
    pills_row.set_homogeneous(false);
    pills_row.set_max_children_per_line(5);
    pills_row.set_min_children_per_line(2);
    pills_row.set_selection_mode(gtk::SelectionMode::None);
    pills_row.set_row_spacing(8);
    pills_row.set_column_spacing(8);
    let step_nav = gtk::Box::new(gtk::Orientation::Vertical, 0);
    step_nav.add_css_class("card");
    step_nav.add_css_class("marius-step-nav");
    let mut step_pills = Vec::new();
    for (i, name) in STEPS.iter().enumerate() {
        let pill = gtk::Button::with_label(&format!("{}. {}", i + 1, step_short_label(i)));
        pill.set_tooltip_text(Some(&format!("{}. {}", i + 1, name)));
        pill.add_css_class("flat");
        pill.add_css_class("marius-pill-step");
        pill.add_css_class("marius-pill-label");
        pill.set_sensitive(i == 0);
        pills_row.append(&pill);
        step_pills.push(pill);
    }
    step_nav.append(&pills_row);

    let step_errors = gtk::Label::new(None);
    step_errors.set_wrap(true);
    step_errors.set_halign(gtk::Align::Start);
    step_errors.set_hexpand(true);
    step_errors.add_css_class("error");
    step_errors.set_visible(false);

    let prev_btn = gtk::Button::with_label("Précédent");
    let next_btn = gtk::Button::with_label("Suivant");
    next_btn.add_css_class("suggested-action");

    let gen_banner = adw::Banner::new("Génération du PDF en cours…");
    gen_banner.set_revealed(false);

    let home_recent_list = gtk::Box::new(gtk::Orientation::Vertical, 6);

    let ui = WizardUi {
        app_stack: app_stack.clone(),
        stack: stack.clone(),
        step_title: step_title.clone(),
        hero_intro: hero.clone(),
        step_scrolled: scrolled.clone(),
        window: window.clone(),
        home_btn: home_btn.clone(),
        save_menu_btn: save_menu_btn.clone(),
        prev_btn: prev_btn.clone(),
        next_btn: next_btn.clone(),
        step_errors: step_errors.clone(),
        step_pills,
        gen_banner: gen_banner.clone(),
        home_recent_list: home_recent_list.clone(),
        last_filled_step: std::cell::Cell::new(None),
    };

    for (i, pill) in ui.step_pills.iter().enumerate() {
        let ui_pill = ui.clone_handles();
        let state_pill = state.clone();
        pill.connect_clicked(move |_| perform_wizard_goto(&ui_pill, &state_pill, i));
    }

    let nav_btns = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    nav_btns.append(&prev_btn);
    nav_btns.append(&next_btn);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    bottom.set_margin_top(12);
    bottom.set_margin_bottom(16);
    bottom.append(&step_errors);
    let bottom_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    bottom_spacer.set_hexpand(true);
    bottom.append(&bottom_spacer);
    bottom.append(&nav_btns);

    let main_column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_column.append(&hero);
    main_column.append(&step_nav);
    main_column.append(&scrolled);
    main_column.append(&gen_banner);
    main_column.append(&bottom);

    let clamp = adw::Clamp::builder()
        .maximum_size(CONTENT_MAX_WIDTH)
        .build();
    clamp.set_margin_start(24);
    clamp.set_margin_end(24);
    clamp.set_child(Some(&main_column));

    wire_btn(&prev_btn, &ui, &state, perform_wizard_prev);
    wire_btn(&next_btn, &ui, &state, |ui, state| {
        perform_wizard_next(ui, state);
    });

    let home_page = home::build_home_page(&state, &ui, &home_recent_list);
    app_stack.add_named(&home_page, Some("home"));
    app_stack.add_named(&clamp, Some("wizard"));
    app_stack.set_visible_child_name("home");
    session::sync_header_chrome(&ui, &state);

    wire_project_window_actions(&window, &ui, &state);

    let ui_home = ui.clone_handles();
    let state_home = state.clone();
    home_btn.connect_clicked(move |_| {
        session::run_with_unsaved_guard(&ui_home, &state_home, |ui, state| {
            session::enter_home(ui, state);
        });
    });

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&app_stack));
    window.set_content(Some(&toolbar_view));

    let ui_close = ui.clone_handles();
    let state_close = state.clone();
    window.connect_close_request(move |_| {
        session::handle_window_close_request(&ui_close, &state_close)
    });

    let state_destroy = state.clone();
    window.connect_destroy(move |_| {
        if let Some(dir) = state_destroy
            .borrow()
            .project
            .as_ref()
            .map(|p| p.workspace_dir.clone())
        {
            remove_ephemeral_workspace(&dir);
        }
        agenda_pipeline::shutdown_pdf_webview();
    });
    window.present();
    ui
}

fn wire_project_window_actions(
    window: &adw::ApplicationWindow,
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
) {
    let action_save = gtk::gio::SimpleAction::new("save", None);
    let ui_save = ui.clone_handles();
    let state_save = state.clone();
    action_save.connect_activate(move |_, _| {
        if state_save.borrow().screen != state::AppScreen::Wizard {
            return;
        }
        let ui_title = ui_save.clone_handles();
        let state_title = state_save.clone();
        session::save_or_prompt_as(&ui_save, &state_save, move || {
            session::update_window_title(&ui_title, &state_title);
        });
    });
    window.add_action(&action_save);

    let action_save_as = gtk::gio::SimpleAction::new("save-as", None);
    let ui_as = ui.clone_handles();
    let state_as = state.clone();
    action_save_as.connect_activate(move |_, _| {
        if state_as.borrow().screen != state::AppScreen::Wizard {
            return;
        }
        let ui_title = ui_as.clone_handles();
        let state_title = state_as.clone();
        session::save_project_as(&ui_as, &state_as, move || {
            session::update_window_title(&ui_title, &state_title);
        });
    });
    window.add_action(&action_save_as);

    let key = gtk::EventControllerKey::new();
    let ui_key = ui.clone_handles();
    let state_key = state.clone();
    key.connect_key_pressed(move |_, keyval, _, modifiers| {
        if !modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            return gtk::glib::Propagation::Proceed;
        }
        if keyval != Key::s {
            return gtk::glib::Propagation::Proceed;
        }
        if state_key.borrow().screen != state::AppScreen::Wizard {
            return gtk::glib::Propagation::Stop;
        }
        let ui_title = ui_key.clone_handles();
        let state_title = state_key.clone();
        session::save_or_prompt_as(&ui_key, &state_key, move || {
            session::update_window_title(&ui_title, &state_title);
        });
        gtk::glib::Propagation::Stop
    });
    window.add_controller(key);
}

fn wire_btn(
    btn: &gtk::Button,
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    f: fn(&WizardUi, &Rc<RefCell<AppState>>),
) {
    let ui = ui.clone_handles();
    let state = state.clone();
    btn.connect_clicked(move |_| f(&ui, &state));
}

/// `force_rebuild` — true after in-step edits (illustrations, toggles); false on step navigation.
pub(crate) fn refresh_step_page(
    ui: &WizardUi,
    state: &Rc<RefCell<AppState>>,
    force_rebuild: bool,
) {
    let step = state.borrow().step;
    ui.hero_intro.set_visible(step == 0);
    ui.step_title.set_text(&format!(
        "Étape {} / {} — {}",
        step + 1,
        WIZARD_STEP_COUNT,
        STEPS[step]
    ));
    let page_id = step.to_string();
    ui.stack.set_visible_child_name(&page_id);

    clear_hidden_step_pages(ui, step);

    let child = ui.stack.child_by_name(&page_id).expect("step page");
    let page = child.downcast_ref::<gtk::Box>().expect("box");
    let page_empty = page.first_child().is_none();
    let need_body = force_rebuild || ui.last_filled_step.get() != Some(step) || page_empty;
    if need_body {
        while let Some(first) = page.first_child() {
            page.remove(&first);
        }

        let hint = gtk::Label::new(None);
        hint.set_wrap(true);
        hint.add_css_class("dim-label");
        hint.add_css_class("marius-step-hint");
        hint.set_halign(gtk::Align::Start);

        steps::fill_step(step, page, &hint, state, ui);
        ui.last_filled_step.set(Some(step));
    }

    ui.next_btn.set_visible(step < WIZARD_STEP_COUNT - 1);
    ui.prev_btn.set_sensitive(step > 0);

    for (i, pill) in ui.step_pills.iter().enumerate() {
        pill.set_sensitive(i <= step);
        pill.set_label(&format!("{}. {}", i + 1, step_short_label(i)));
        if i == step {
            pill.add_css_class("suggested-action");
            pill.remove_css_class("flat");
        } else {
            pill.remove_css_class("suggested-action");
            pill.add_css_class("flat");
        }
    }

    update_wizard_errors(ui, state);
    reset_step_scroll(ui);
    session::update_window_title(ui, state);
}

fn reset_step_scroll(ui: &WizardUi) {
    let scrolled = ui.step_scrolled.clone();
    glib::idle_add_local_once(move || {
        let adj = scrolled.vadjustment();
        adj.set_value(adj.lower());
    });
}

fn clear_hidden_step_pages(ui: &WizardUi, visible_step: usize) {
    for i in 0..WIZARD_STEP_COUNT {
        if i == visible_step {
            continue;
        }
        let child = ui.stack.child_by_name(&i.to_string()).expect("step page");
        let page = child.downcast_ref::<gtk::Box>().expect("box");
        while let Some(first) = page.first_child() {
            page.remove(&first);
        }
        if ui.last_filled_step.get() == Some(i) {
            ui.last_filled_step.set(None);
        }
    }
}

/// Block wizard navigation and step body while PDF export runs (banner stays visible).
pub(crate) fn set_wizard_busy(ui: &WizardUi, state: &Rc<RefCell<AppState>>, busy: bool) {
    if busy {
        ui.hero_intro.set_sensitive(false);
        ui.step_scrolled.set_sensitive(false);
        ui.prev_btn.set_sensitive(false);
        ui.next_btn.set_sensitive(false);
        ui.home_btn.set_sensitive(false);
        ui.save_menu_btn.set_sensitive(false);
        for pill in &ui.step_pills {
            pill.set_sensitive(false);
        }
        return;
    }
    ui.hero_intro.set_sensitive(true);
    ui.step_scrolled.set_sensitive(true);
    let step = state.borrow().step;
    ui.prev_btn.set_sensitive(step > 0);
    session::sync_header_chrome(ui, state);
    for (i, pill) in ui.step_pills.iter().enumerate() {
        pill.set_sensitive(i <= step);
    }
    update_wizard_errors(ui, state);
}

pub(crate) fn update_wizard_errors(ui: &WizardUi, state: &Rc<RefCell<AppState>>) {
    let s = state.borrow();
    let errs = validate_wizard_step(s.step, &s.config);
    let on_last = s.step >= WIZARD_STEP_COUNT - 1;
    if errs.is_empty() {
        ui.step_errors.set_visible(false);
        if !on_last {
            ui.next_btn.set_sensitive(true);
        }
    } else {
        ui.step_errors.set_text(&format_wizard_errors(&errs));
        ui.step_errors.set_visible(true);
        ui.next_btn.set_sensitive(false);
    }
}
