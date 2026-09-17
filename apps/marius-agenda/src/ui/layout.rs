//! Shared GTK layout helpers (Libadwaita patterns).

use super::date_field::{date_field, DateField};
use agenda_core::IllustrationSlot;
use gtk::prelude::*;
use gtk::{Align, Box as GtkBox, Entry, Grid, Label, Orientation, Picture};
use std::path::{Path, PathBuf};

pub const CONTENT_MAX_WIDTH: i32 = 880;

pub fn load_app_css() {
    let provider = gtk::CssProvider::new();
    let css = include_str!("../../ui/marius.css");
    provider.load_from_string(css);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn req_legend(text: &str) -> Label {
    let lbl = Label::new(Some(text));
    lbl.set_halign(Align::Start);
    lbl.add_css_class("dim-label");
    lbl.add_css_class("marius-req-legend");
    lbl
}

pub fn req_legend_star_lead(rest: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 4);
    row.set_halign(Align::Start);
    row.add_css_class("marius-req-legend");
    let star = Label::new(Some("*"));
    star.add_css_class("marius-required");
    let lbl = Label::new(Some(rest));
    lbl.add_css_class("dim-label");
    row.append(&star);
    row.append(&lbl);
    row
}

pub fn field_label_required(text: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 4);
    row.set_halign(Align::Start);
    let lbl = Label::new(Some(text));
    let star = Label::new(Some("*"));
    star.add_css_class("marius-required");
    row.append(&lbl);
    row.append(&star);
    row
}

pub fn two_column_grid(parent: &GtkBox) -> Grid {
    let grid = Grid::new();
    grid.set_column_spacing(16);
    grid.set_row_spacing(16);
    grid.set_column_homogeneous(true);
    parent.append(&grid);
    grid
}

pub fn grid_field(grid: &Grid, col: i32, row: i32, label: &str, value: &str) -> Entry {
    let col_box = GtkBox::new(Orientation::Vertical, 4);
    col_box.append(&field_label_required(label.trim_end_matches(" *").trim_end_matches('*')));
    let entry = Entry::new();
    entry.set_text(value);
    col_box.append(&entry);
    grid.attach(&col_box, col, row, 1, 1);
    entry
}

pub fn grid_date_field(grid: &Grid, col: i32, row: i32, label: &str, value: &str) -> DateField {
    let col_box = GtkBox::new(Orientation::Vertical, 4);
    col_box.append(&field_label_required(label));
    let field = date_field(value);
    col_box.append(field.widget());
    let hint = Label::new(Some("Saisie ou calendrier"));
    hint.add_css_class("caption");
    hint.add_css_class("dim-label");
    hint.set_halign(Align::Start);
    col_box.append(&hint);
    grid.attach(&col_box, col, row, 1, 1);
    field
}

pub fn section_heading(title: &str, desc: &str) -> GtkBox {
    let block = GtkBox::new(Orientation::Vertical, 8);
    let title_lbl = Label::new(Some(title));
    title_lbl.add_css_class("heading");
    title_lbl.set_halign(Align::Start);
    title_lbl.set_margin_top(8);
    block.append(&title_lbl);
    let desc_lbl = Label::new(Some(desc));
    desc_lbl.set_wrap(true);
    desc_lbl.add_css_class("dim-label");
    desc_lbl.set_halign(Align::Start);
    desc_lbl.set_margin_bottom(4);
    block.append(&desc_lbl);
    block
}

pub fn holiday_table_grid(parent: &GtkBox) -> Grid {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(10);
    grid.set_column_homogeneous(false);
    grid.set_margin_top(4);
    grid.set_margin_bottom(4);
    parent.append(&grid);
    grid
}

pub fn holiday_header_row(grid: &Grid, row: i32) {
    for (col, text) in [(0, "Nom"), (1, "Début"), (2, "Fin")] {
        let lbl = Label::new(Some(text));
        lbl.add_css_class("heading");
        lbl.set_halign(Align::Start);
        lbl.set_hexpand(col == 0);
        grid.attach(&lbl, col, row, 1, 1);
    }
    let spacer = Label::new(None);
    grid.attach(&spacer, 3, row, 1, 1);
}

pub struct IlluTileGrid {
    pub grid: Grid,
    col: i32,
    row: i32,
    cols: i32,
}

impl IlluTileGrid {
    pub fn new(cols: i32) -> Self {
        let grid = Grid::new();
        grid.set_column_spacing(16);
        grid.set_row_spacing(16);
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(false);
        grid.set_halign(Align::Start);
        grid.add_css_class("marius-illu-grid");
        Self {
            grid,
            col: 0,
            row: 0,
            cols,
        }
    }

    pub fn next_slot(&mut self) -> (i32, i32) {
        let slot = (self.col, self.row);
        self.col += 1;
        if self.col >= self.cols {
            self.col = 0;
            self.row += 1;
        }
        slot
    }
}

pub fn step_card(page: &GtkBox) -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 16);
    card.add_css_class("card");
    card.add_css_class("marius-step-card");
    card.set_margin_top(4);
    page.append(&card);
    card
}

pub fn trash_icon_button(tooltip: &str) -> gtk::Button {
    let btn = gtk::Button::from_icon_name("user-trash-symbolic");
    btn.set_tooltip_text(Some(tooltip));
    btn.add_css_class("destructive-action");
    btn.add_css_class("flat");
    btn
}

pub fn delete_holiday_button() -> gtk::Button {
    trash_icon_button("Supprimer cette période de vacances")
}

pub fn segmented_mode_box() -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("linked");
    row.add_css_class("marius-mode-box");
    row
}

pub fn slot_preview_path(root: &Path, data_dir: &Path, slot: &IllustrationSlot) -> Option<PathBuf> {
    if let Some(ref rel) = slot.print_path {
        let abs = data_dir.join(rel);
        if abs.is_file() {
            return Some(abs);
        }
    }
    if let Some(ref rel) = slot.source_path {
        let p = Path::new(rel);
        if p.is_absolute() && p.is_file() {
            return Some(p.to_path_buf());
        }
        for candidate in [root.join(rel), data_dir.join(rel)] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn set_picture_path(pic: &Picture, path: Option<&Path>) {
    if let Some(path) = path.filter(|p| p.is_file()) {
        if let Ok(texture) = gtk::gdk::Texture::from_filename(path) {
            pic.set_paintable(Some(&texture));
            return;
        }
        pic.set_filename(Some(path));
        return;
    }
    pic.set_paintable(None::<&gtk::gdk::Paintable>);
}

pub fn illustration_preview(path: Option<&Path>) -> Picture {
    let pic = Picture::new();
    pic.set_content_fit(gtk::ContentFit::Contain);
    pic.set_size_request(140, 140);
    pic.add_css_class("marius-illu-preview");
    set_picture_path(&pic, path);
    pic
}

pub fn output_folder_panel(path_label: &str, on_pick: std::rc::Rc<dyn Fn()>) -> GtkBox {
    let wrap = GtkBox::new(Orientation::Vertical, 10);
    wrap.add_css_class("marius-output-folder");

    let title = Label::new(Some("Dossier d’enregistrement"));
    title.add_css_class("heading");
    title.set_halign(Align::Start);
    wrap.append(&title);

    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("marius-output-folder-row");
    row.set_valign(gtk::Align::Center);

    let path = Label::new(Some(path_label));
    path.set_wrap(true);
    path.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    path.set_xalign(0.0);
    path.set_hexpand(true);
    path.set_selectable(true);
    path.add_css_class("marius-output-path");
    row.append(&path);

    let btn = gtk::Button::with_label("Choisir…");
    btn.add_css_class("suggested-action");
    btn.connect_clicked(move |_| on_pick());
    row.append(&btn);

    wrap.append(&row);
    wrap
}
