//! French national phone entry (10 digits, no country code).

use agenda_core::format_fr_phone_live;
use gtk::prelude::*;
use gtk::Entry;
use std::cell::Cell;

pub fn attach_fr_phone_entry(entry: &Entry) {
    entry.set_placeholder_text(Some("06 12 34 56 78"));
    entry.set_max_width_chars(14);
    entry.set_input_purpose(gtk::InputPurpose::Phone);

    let adjusting = Cell::new(false);
    entry.connect_changed(move |e| {
        if adjusting.get() {
            return;
        }
        let raw = e.text();
        let formatted = format_fr_phone_live(raw.as_str());
        if formatted == raw.as_str() {
            return;
        }
        adjusting.set(true);
        e.set_text(&formatted);
        e.set_position(-1);
        adjusting.set(false);
    });
}

pub fn phone_entry_text(entry: &Entry) -> String {
    format_fr_phone_live(entry.text().as_str())
}
