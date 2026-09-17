use super::generate::generate_period;
use super::layout::{
    delete_holiday_button, grid_date_field, grid_field, holiday_header_row, holiday_table_grid,
    illustration_preview, IlluTileGrid,
    output_folder_panel, req_legend, req_legend_star_lead, section_heading,
    segmented_mode_box,
    slot_preview_path, step_card, two_column_grid,
};
use super::navigation::WizardUi;
use super::state::{apply_step_sync, edit_state_then, AppState};
use super::util::{confirm_destructive, on_click_refresh, pick_image_file, pick_output_folder};
use super::phone_field::{attach_fr_phone_entry, phone_entry_text};
use agenda_core::{
    empty_contact, empty_slot, list_school_periods, AgendaConfig, AgendaContact, BookletDuplexPass,
    HolidayPeriod, IllustrationSlot, MAX_CONTACT_PHONES, weekday_label_fr, WEEKDAY_ORDER,
};
use agenda_images::{import_illustration_file, IllustrationSlotId};
use gtk::prelude::*;
use gtk::{CheckButton, Entry, FlowBox, Label, Orientation};
use gtk::Box as GtkBox;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

fn project_data_dir(state: &Rc<RefCell<AppState>>) -> PathBuf {
    state
        .borrow()
        .workspace_dir()
        .cloned()
        .expect("active project workspace")
}

pub fn fill_step(
    step: usize,
    page: &GtkBox,
    hint: &Label,
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
) {
    state.borrow().set_sync(None);
    let config = state.borrow().config.clone();
    match step {
        0 => fill_year(page, hint, &config, state, ui),
        1 => fill_holidays(page, hint, state, ui),
        2 => fill_weekdays(page, hint, state),
        3 => fill_illustrations(page, hint, state, ui),
        4 => fill_generate(page, hint, state, ui),
        _ => {}
    }
}

fn fill_year(
    page: &GtkBox,
    hint: &Label,
    config: &AgendaConfig,
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
) {
    hint.set_text(
        "Renseigne le titre, l’année scolaire, les dates clés et les contacts à afficher en fin d’agenda.",
    );
    page.append(hint);
    page.append(&req_legend("Les champs généraux sont obligatoires. Les contacts sont optionnels."));

    let card = step_card(page);
    let grid = two_column_grid(&card);
    let title = grid_field(&grid, 0, 0, "Titre", &config.title);
    let year = grid_field(&grid, 1, 0, "Année scolaire", &config.school_year_label);
    let rentree = grid_date_field(&grid, 0, 1, "Rentrée", &config.rentree);
    let fin = grid_date_field(&grid, 1, 1, "Fin des cours", &config.fin_des_cours);

    page.append(&section_heading(
        "Contacts",
        "Nom et numéro national obligatoires pour chaque contact (10 chiffres, sans indicatif) — maximum 10.",
    ));
    let contacts_card = step_card(page);
    let contacts_box = GtkBox::new(Orientation::Vertical, 8);
    contacts_card.append(&contacts_box);

    let initial: Vec<AgendaContact> = if config.contacts.is_empty() {
        vec![empty_contact()]
    } else {
        config.contacts.clone()
    };
    let mut contact_rows: Vec<(Entry, Entry)> = Vec::new();
    for contact in initial.iter().take(MAX_CONTACT_PHONES) {
        let row = GtkBox::new(Orientation::Horizontal, 8);
        let name = Entry::new();
        name.set_text(&contact.name);
        name.set_placeholder_text(Some("Nom (obligatoire)"));
        name.set_width_chars(18);
        name.set_hexpand(true);
        let phone = Entry::new();
        phone.set_text(&contact.phone);
        attach_fr_phone_entry(&phone);
        phone.set_hexpand(true);
        row.append(&name);
        row.append(&phone);
        contacts_box.append(&row);
        contact_rows.push((name, phone));
    }

    let add = gtk::Button::with_label("+ Ajouter un contact");
    add.set_halign(gtk::Align::Start);
    add.set_sensitive(contact_rows.len() < MAX_CONTACT_PHONES);
    add.connect_clicked(on_click_refresh(state, ui, |state| {
        apply_step_sync(state);
        let mut s = state.borrow_mut();
        let row_count = if s.config.contacts.is_empty() {
            1
        } else {
            s.config.contacts.len()
        };
        if row_count < MAX_CONTACT_PHONES {
            s.config.contacts.push(empty_contact());
        }
        s.mark_project_dirty();
    }));
    contacts_card.append(&add);

    state.borrow().set_sync(Some(Box::new(move |cfg: &mut AgendaConfig| {
        cfg.title = title.text().to_string();
        cfg.school_year_label = year.text().to_string();
        cfg.rentree = rentree.text();
        cfg.fin_des_cours = fin.text();
        cfg.contacts = contact_rows
            .iter()
            .map(|(name, phone)| AgendaContact {
                name: name.text().to_string(),
                phone: phone_entry_text(phone),
            })
            .take(MAX_CONTACT_PHONES)
            .collect();
    })));
}

fn fill_holidays(page: &GtkBox, hint: &Label, state: &Rc<RefCell<AppState>>, ui: &WizardUi) {
    hint.set_text("Pour chaque plage, indique le nom et les dates de début et de fin.");
    page.append(hint);

    let card = step_card(page);
    let grid = holiday_table_grid(&card);
    holiday_header_row(&grid, 0);

    let holidays = state.borrow().config.holidays.clone();
    let mut rows: Vec<(Entry, super::date_field::DateField, super::date_field::DateField)> = Vec::new();
    for (idx, h) in holidays.iter().enumerate() {
        let (label, start, end) = holiday_fields(
            &grid,
            idx as i32 + 1,
            h,
            h.id.clone(),
            state,
            ui,
        );
        rows.push((label, start, end));
    }

    let add = gtk::Button::with_label("+ Ajouter des vacances");
    add.set_margin_top(12);
    add.set_halign(gtk::Align::Start);
    add.connect_clicked(on_click_refresh(state, ui, |state| {
        apply_step_sync(state);
        let mut s = state.borrow_mut();
        let rentree = s.config.rentree.clone();
        s.config.holidays.push(HolidayPeriod {
            id: format!("vacances-{}", unix_nanos()),
            label: "Nouvelles vacances".into(),
            start: rentree.clone(),
            end: rentree,
        });
        s.mark_project_dirty();
    }));
    card.append(&add);

    let state_sync = state.clone();
    state_sync.borrow().set_sync(Some(Box::new(move |cfg: &mut AgendaConfig| {
        for (i, (label, start, end)) in rows.iter().enumerate() {
            if let Some(h) = cfg.holidays.get_mut(i) {
                h.label = label.text().to_string();
                h.start = start.text();
                h.end = end.text();
            }
        }
    })));
}

fn holiday_fields(
    grid: &gtk::Grid,
    row: i32,
    h: &HolidayPeriod,
    holiday_id: String,
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
) -> (Entry, super::date_field::DateField, super::date_field::DateField) {
    let label = Entry::new();
    label.set_text(&h.label);
    label.set_hexpand(true);
    let start = super::date_field::date_field(&h.start);
    let end = super::date_field::date_field(&h.end);
    let rm = delete_holiday_button();
    let window = ui.window.clone();
    let state = state.clone();
    let ui = ui.clone_handles();
    rm.connect_clicked(move |_| {
        let holiday_id = holiday_id.clone();
        let state_confirm = state.clone();
        let ui_confirm = ui.clone_handles();
        confirm_destructive(
            &window,
            "Supprimer ces vacances ?",
            "Cette suppression est définitive dans ta configuration.",
            "Supprimer",
            move || {
                apply_step_sync(&state_confirm);
                edit_state_then(
                    &state_confirm,
                    |s| {
                        s.config.holidays.retain(|h| h.id != holiday_id);
                        s.mark_project_dirty();
                    },
                    || super::refresh_step_page(&ui_confirm, &state_confirm, true),
                );
            },
        );
    });
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(start.widget(), 1, row, 1, 1);
    grid.attach(end.widget(), 2, row, 1, 1);
    grid.attach(&rm, 3, row, 1, 1);
    rm.set_valign(gtk::Align::Center);
    (label, start, end)
}

fn fill_weekdays(page: &GtkBox, hint: &Label, state: &Rc<RefCell<AppState>>) {
    hint.set_text("Coche les jours qui auront une page dans l’agenda.");
    page.append(hint);
    page.append(&req_legend_star_lead("Au moins un jour obligatoire"));

    let card = step_card(page);
    let flow = FlowBox::new();
    flow.set_homogeneous(false);
    flow.set_max_children_per_line(4);
    flow.set_min_children_per_line(2);
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_margin_top(4);

    let active = state.borrow().config.school_days.clone();
    for wd in WEEKDAY_ORDER {
        let row = CheckButton::with_label(weekday_label_fr(wd));
        row.set_active(active.get(wd));
        row.set_margin_top(6);
        row.set_margin_bottom(6);
        row.set_margin_start(4);
        row.set_margin_end(4);
        let state = state.clone();
        row.connect_toggled(move |btn| {
            let mut s = state.borrow_mut();
            s.config.school_days.set(wd, btn.is_active());
            s.mark_project_dirty();
        });
        flow.append(&row);
    }
    card.append(&flow);
}

fn fill_illustrations(page: &GtkBox, hint: &Label, state: &Rc<RefCell<AppState>>, ui: &WizardUi) {
    hint.set_text(
        "Ajoute tes illustrations : elles seront préparées en noir et blanc pour l’impression. Tu peux passer cette étape si les images par défaut te conviennent.",
    );
    page.append(hint);

    let root = state.borrow().root.clone();
    let data_dir = project_data_dir(state);
    let cfg = state.borrow().config.clone();

    let card_start = step_card(page);
    card_start.append(&section_heading(
        "Début de l’agenda",
        "Première de couverture et page des activités de la semaine.",
    ));
    let mut start_grid = IlluTileGrid::new(3);
    for (title, slot, id) in [
        (
            "Couverture",
            &cfg.illustrations.cover,
            IllustrationSlotId::Cover,
        ),
        (
            "Activités",
            &cfg.illustrations.activities,
            IllustrationSlotId::Activities,
        ),
    ] {
        let (col, row) = start_grid.next_slot();
        illu_tile(
            &start_grid.grid,
            col,
            row,
            title,
            slot,
            &root,
            &data_dir,
            state,
            ui,
            id,
        );
    }
    card_start.append(&start_grid.grid);

    let card_days = step_card(page);
    card_days.append(&section_heading(
        "Pages des jours",
        "Dessin affiché en bas de chaque page du jour correspondant.",
    ));
    let mut days_grid = IlluTileGrid::new(3);
    for wd in WEEKDAY_ORDER {
        if !cfg.school_days.get(wd) {
            continue;
        }
        let slot = cfg
            .illustrations
            .weekday_slot(wd)
            .cloned()
            .unwrap_or_else(empty_slot);
        let (col, row) = days_grid.next_slot();
        illu_tile(
            &days_grid.grid,
            col,
            row,
            weekday_label_fr(wd),
            &slot,
            &root,
            &data_dir,
            state,
            ui,
            IllustrationSlotId::Weekday(wd),
        );
    }
    card_days.append(&days_grid.grid);
}

fn illu_tile(
    grid: &gtk::Grid,
    col: i32,
    row: i32,
    title: &str,
    slot: &IllustrationSlot,
    root: &PathBuf,
    data_dir: &PathBuf,
    state: &Rc<RefCell<AppState>>,
    ui: &WizardUi,
    id: IllustrationSlotId,
) {
    let block = GtkBox::new(Orientation::Vertical, 6);
    block.add_css_class("card");
    block.add_css_class("marius-illu-tile");
    block.set_halign(gtk::Align::Center);
    block.set_valign(gtk::Align::Start);
    block.set_hexpand(false);
    block.set_vexpand(false);
    let title_lbl = Label::new(Some(title));
    title_lbl.set_halign(gtk::Align::Center);
    block.append(&title_lbl);
    let preview_path = slot_preview_path(root, data_dir, slot);
    block.append(&illustration_preview(preview_path.as_deref()));
    let caption = slot_caption(slot);
    let cap = Label::new(Some(&caption));
    cap.set_wrap(true);
    cap.set_max_width_chars(20);
    cap.add_css_class("caption");
    cap.add_css_class("dim-label");
    cap.set_halign(gtk::Align::Center);
    block.append(&cap);
    let btn = gtk::Button::with_label("Choisir…");
    btn.set_halign(gtk::Align::Center);
    btn.set_hexpand(false);
    let window = ui.window.clone();
    let state = state.clone();
    let ui = ui.clone_handles();
    btn.connect_clicked(move |_| {
        let state = state.clone();
        let ui = ui.clone_handles();
        pick_image_file(&window, move |path| {
            if let Some(path) = path {
                let root = state.borrow().root.clone();
                let data_dir = project_data_dir(&state);
                edit_state_then(
                    &state,
                    |s| match import_illustration_file(&root, &data_dir, &s.config, id, &path) {
                        Ok(cfg) => {
                            s.config = cfg;
                            s.mark_project_dirty();
                        }
                        Err(e) => eprintln!("Image: {}", e),
                    },
                    || super::refresh_step_page(&ui, &state, true),
                );
            }
        });
    });
    block.append(&btn);
    grid.attach(&block, col, row, 1, 1);
}

fn slot_caption(slot: &IllustrationSlot) -> String {
    if slot.source_path.is_none() && slot.print_path.is_none() {
        return "Image par défaut".into();
    }
    slot.print_path
        .as_deref()
        .or(slot.source_path.as_deref())
        .map(|p| {
            let name = std::path::Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.to_string());
            format!("{}", name)
        })
        .unwrap_or_else(|| "Image par défaut".into())
}

fn fill_generate(page: &GtkBox, hint: &Label, state: &Rc<RefCell<AppState>>, ui: &WizardUi) {
    hint.set_text("Choisis le format du PDF, puis lance la génération pour une période.");
    page.append(hint);

    let card = step_card(page);
    card.add_css_class("marius-generate-card");
    let imposed = state.borrow().config.booklet.generate_imposed_pdf;
    let ignore_mode_toggle = Rc::new(Cell::new(true));
    let mode_row = segmented_mode_box();
    let page_btn = gtk::ToggleButton::with_label("Page à page");
    let booklet_btn = gtk::ToggleButton::with_label("Livret");
    page_btn.set_group(Some(&booklet_btn));
    mode_row.append(&page_btn);
    mode_row.append(&booklet_btn);
    card.append(&mode_row);

    let desc = Label::new(None);
    desc.set_wrap(true);
    desc.add_css_class("dim-label");
    desc.set_halign(gtk::Align::Start);
    desc.add_css_class("marius-generate-section");
    desc.set_margin_top(12);
    desc.set_margin_bottom(12);
    card.append(&desc);

    let duplex_expander = append_duplex_expander(&card, state);
    duplex_expander.set_visible(imposed);

    wire_booklet_mode(
        &page_btn,
        state,
        false,
        &duplex_expander,
        &desc,
        ignore_mode_toggle.clone(),
    );
    wire_booklet_mode(
        &booklet_btn,
        state,
        true,
        &duplex_expander,
        &desc,
        ignore_mode_toggle.clone(),
    );

    if imposed {
        booklet_btn.set_active(true);
    } else {
        page_btn.set_active(true);
    }
    apply_booklet_mode_labels(imposed, &desc);
    ignore_mode_toggle.set(false);

    let output_dir = state.borrow().config.output_dir.clone();
    let folder_subtitle = if output_dir.trim().is_empty() {
        "Aucun dossier choisi"
    } else {
        output_dir.as_str()
    };
    let state_pick = state.clone();
    let ui_pick = ui.clone_handles();
    let window = ui.window.clone();
    let pick_folder: Rc<dyn Fn()> = {
        let state_pick = state_pick.clone();
        let ui_pick = ui_pick.clone_handles();
        let window = window.clone();
        Rc::new(move || {
            let last = state_pick.borrow().config.output_dir.clone();
            let state_pick = state_pick.clone();
            let ui_pick = ui_pick.clone_handles();
            pick_output_folder(&window, &last, move |picked| {
                if let Some(picked) = picked {
                    if let Err(e) =
                        super::session::set_project_output_dir(&ui_pick, &state_pick, &picked)
                    {
                        super::util::message(
                            &ui_pick.window,
                            "Dossier de sortie",
                            &e.to_string(),
                        );
                        return;
                    }
                    super::refresh_step_page(&ui_pick, &state_pick, true);
                }
            });
        })
    };
    card.append(&output_folder_panel(folder_subtitle, pick_folder));

    let periods = list_school_periods(&state.borrow().config);
    if periods.is_empty() {
        card.append(&Label::new(Some(
            "Aucune période — vérifie rentrée, fin des cours, vacances et jours cochés.",
        )));
        return;
    }

    let periods_card = step_card(page);
    let periods_title = Label::new(Some("Périodes scolaires"));
    periods_title.add_css_class("heading");
    periods_title.set_halign(gtk::Align::Start);
    periods_card.append(&periods_title);

    let flow = FlowBox::new();
    flow.set_homogeneous(false);
    flow.set_max_children_per_line(3);
    flow.set_min_children_per_line(1);
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_margin_top(8);
    flow.set_column_spacing(12);
    flow.set_row_spacing(12);

    for (idx, p) in periods.iter().enumerate() {
        let (card, gen_btn) = period_card(idx, p);
        let state = state.clone();
        let ui = ui.clone_handles();
        let pid = p.id.clone();
        let plabel = p.label.clone();
        gen_btn.connect_clicked(move |_| generate_period(&ui, &state, &pid, &plabel));
        flow.append(&card);
    }
    periods_card.append(&flow);
}

fn period_card(idx: usize, p: &agenda_core::SchoolPeriod) -> (GtkBox, gtk::Button) {
    let card = GtkBox::new(Orientation::Vertical, 8);
    card.add_css_class("card");
    card.add_css_class("marius-period-card");
    card.set_width_request(300);

    let top = GtkBox::new(Orientation::Horizontal, 10);
    let index = Label::new(Some(&format!("{:02}", idx + 1)));
    index.add_css_class("marius-period-index");
    index.set_valign(gtk::Align::Start);
    top.append(&index);

    let meta = GtkBox::new(Orientation::Vertical, 2);
    meta.set_hexpand(true);
    let title = Label::new(Some(&p.label));
    title.set_wrap(true);
    title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    title.set_halign(gtk::Align::Start);
    title.add_css_class("heading");
    meta.append(&title);
    let dates = Label::new(Some(&format!("{} → {}", p.start, p.end)));
    dates.add_css_class("dim-label");
    dates.set_halign(gtk::Align::Start);
    meta.append(&dates);
    let days = Label::new(Some(&format!("{} jours", p.days.len())));
    days.add_css_class("caption");
    days.set_halign(gtk::Align::Start);
    meta.append(&days);
    top.append(&meta);
    card.append(&top);

    let gen_btn = gtk::Button::with_label("Créer le PDF");
    gen_btn.add_css_class("suggested-action");
    gen_btn.set_halign(gtk::Align::Fill);
    gen_btn.set_hexpand(true);
    card.append(&gen_btn);
    (card, gen_btn)
}

fn apply_booklet_mode_labels(imposed: bool, desc: &Label) {
    let text = if imposed {
        "Livret : deux pages par feuille A4, à imprimer recto-verso puis plier et agrafer au milieu."
    } else {
        "Page à page : une page d’agenda par feuille, pratique pour lire à l’écran."
    };
    desc.set_text(text);
}

fn wire_booklet_mode(
    btn: &gtk::ToggleButton,
    state: &Rc<RefCell<AppState>>,
    imposed: bool,
    duplex_expander: &gtk::Expander,
    mode_desc: &Label,
    ignore_toggle: Rc<Cell<bool>>,
) {
    let state = state.clone();
    let duplex = duplex_expander.clone();
    let mode_desc = mode_desc.clone();
    btn.connect_toggled(move |b| {
        if ignore_toggle.get() || !b.is_active() {
            return;
        }
        let mut s = state.borrow_mut();
        s.config.booklet.generate_imposed_pdf = imposed;
        s.mark_project_dirty();
        duplex.set_visible(imposed);
        apply_booklet_mode_labels(imposed, &mode_desc);
    });
}

fn append_duplex_expander(page: &GtkBox, state: &Rc<RefCell<AppState>>) -> gtk::Expander {
    let expander = gtk::Expander::new(Some("Imprimante sans duplex automatique"));
    expander.add_css_class("marius-duplex-expander");
    expander.set_use_markup(false);
    let body = GtkBox::new(Orientation::Vertical, 8);
    append_duplex_body(&body, state);
    expander.set_child(Some(&body));
    page.append(&expander);
    expander
}

fn append_duplex_body(page: &GtkBox, state: &Rc<RefCell<AppState>>) {
    let intro = Label::new(Some(
        "Choisis quelles faces du livret générer si ton imprimante ne fait pas le recto-verso toute seule.",
    ));
    intro.set_wrap(true);
    intro.add_css_class("dim-label");
    intro.set_halign(gtk::Align::Start);
    page.append(&intro);

    let pass = state.borrow().config.booklet.duplex_pass;
    let duplex = segmented_mode_box();
    let mut leader: Option<gtk::ToggleButton> = None;
    for (label, value) in [
        ("Les deux", BookletDuplexPass::Both),
        ("Impaires", BookletDuplexPass::Odd),
        ("Paires", BookletDuplexPass::Even),
    ] {
        let btn = gtk::ToggleButton::with_label(label);
        if let Some(ref g) = leader {
            btn.set_group(Some(g));
        } else {
            leader = Some(btn.clone());
        }
        if pass == value {
            btn.set_active(true);
        }
        let state = state.clone();
        btn.connect_toggled(move |b| {
            if !b.is_active() {
                return;
            }
            let mut s = state.borrow_mut();
            s.config.booklet.duplex_pass = value;
            s.mark_project_dirty();
        });
        duplex.append(&btn);
    }
    page.append(&duplex);

    let steps = Label::new(Some(
        "1. Clique sur Impaires, génère la période, imprime le PDF en une seule face.\n\
2. Garde la pile telle quelle (souvent vierge dessus) et remets-la dans le bac.\n\
3. Clique sur Paires, regénère la même période, imprime le second PDF.\n\
4. Si ton imprimante fait le recto-verso : laisse « Les deux » et retournement sur le bord court.",
    ));
    steps.set_wrap(true);
    steps.set_xalign(0.0);
    steps.add_css_class("dim-label");
    steps.set_margin_top(8);
    page.append(&steps);
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}
