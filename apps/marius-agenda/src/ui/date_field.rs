//! ISO date entry with GtkCalendar popover (YYYY-MM-DD).

use agenda_core::parse_iso_date;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Calendar, Entry, MenuButton, Orientation, Popover};

pub struct DateField {
    pub entry: Entry,
    row: GtkBox,
    #[allow(dead_code)]
    popover: Popover,
}

impl DateField {
    pub fn text(&self) -> String {
        self.entry.text().to_string()
    }

    pub fn widget(&self) -> &GtkBox {
        &self.row
    }
}

pub fn date_field(iso: &str) -> DateField {
    let row = GtkBox::new(Orientation::Horizontal, 4);
    let entry = Entry::new();
    entry.set_text(iso);
    entry.set_width_chars(11);
    entry.set_hexpand(true);

    let pick = MenuButton::new();
    pick.set_icon_name("x-office-calendar-symbolic");
    pick.set_tooltip_text(Some("Choisir une date"));
    pick.add_css_class("flat");

    let popover = Popover::new();
    pick.set_popover(Some(&popover));
    let calendar = Calendar::new();
    if let Some(dt) = iso_to_gdatetime(iso) {
        calendar.select_day(&dt);
    }
    popover.set_child(Some(&calendar));

    let entry_sync = entry.clone();
    let popdown = popover.clone();
    calendar.connect_day_selected(move |cal| {
        entry_sync.set_text(&gdatetime_to_iso(&cal.date()));
        popdown.popdown();
    });

    row.append(&entry);
    row.append(&pick);

    DateField { entry, row, popover }
}

fn iso_to_gdatetime(iso: &str) -> Option<gtk::glib::DateTime> {
    let date = parse_iso_date(iso).ok()?;
    gtk::glib::DateTime::from_local(
        date.year(),
        date.month() as i32,
        date.day() as i32,
        0,
        0,
        0.0,
    )
    .ok()
}

fn gdatetime_to_iso(dt: &gtk::glib::DateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        dt.year(),
        dt.month(),
        dt.day_of_month()
    )
}
