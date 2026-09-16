use crate::config::{
    AgendaConfig, AgendaIllustrations, BookletDuplexPass, BookletOptions, HolidayPeriod,
    IllustrationSlot, SchoolDaysMap, WeekdayIndex, empty_slot,
};
use std::collections::HashMap;

pub const WEEKDAY_LABELS_FR: [(WeekdayIndex, &str); 7] = [
    (0, "Dimanche"),
    (1, "Lundi"),
    (2, "Mardi"),
    (3, "Mercredi"),
    (4, "Jeudi"),
    (5, "Vendredi"),
    (6, "Samedi"),
];

/// Mon → Sun display order for UI / activities.
pub const WEEKDAY_ORDER: [WeekdayIndex; 7] = [1, 2, 3, 4, 5, 6, 0];

pub fn weekday_label_fr(wd: WeekdayIndex) -> &'static str {
    WEEKDAY_LABELS_FR
        .iter()
        .find(|(id, _)| *id == wd)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

fn empty_weekday_slots() -> HashMap<String, IllustrationSlot> {
    let mut m = HashMap::new();
    for wd in 0..7u8 {
        m.insert(wd.to_string(), empty_slot());
    }
    m
}

fn empty_illustrations() -> AgendaIllustrations {
    AgendaIllustrations {
        cover: empty_slot(),
        activities: empty_slot(),
        by_weekday: empty_weekday_slots(),
    }
}

/// New GUI project: default school-year preset, no bundled illustration paths.
pub fn create_blank_project_config() -> AgendaConfig {
    let mut config = create_default_config();
    config.illustrations = empty_illustrations();
    config
}

/// Default preset: school year 2026-2027, no bundled illustrations.
pub fn create_default_config() -> AgendaConfig {
    AgendaConfig {
        version: 1,
        title: "Mon agenda".into(),
        school_year_label: "2026-2027".into(),
        rentree: "2026-09-01".into(),
        fin_des_cours: "2027-07-03".into(),
        holidays: vec![
            HolidayPeriod {
                id: "toussaint".into(),
                label: "Vacances de la Toussaint".into(),
                start: "2026-10-17".into(),
                end: "2026-11-01".into(),
            },
            HolidayPeriod {
                id: "noel".into(),
                label: "Vacances de Noël".into(),
                start: "2026-12-19".into(),
                end: "2027-01-03".into(),
            },
            HolidayPeriod {
                id: "hiver".into(),
                label: "Vacances d'hiver".into(),
                start: "2027-02-13".into(),
                end: "2027-02-28".into(),
            },
            HolidayPeriod {
                id: "printemps".into(),
                label: "Vacances de printemps".into(),
                start: "2027-04-10".into(),
                end: "2027-04-25".into(),
            },
        ],
        school_days: SchoolDaysMap {
            days: [false, true, true, false, true, true, false],
        },
        illustrations: empty_illustrations(),
        booklet: BookletOptions {
            generate_imposed_pdf: true,
            duplex_pass: BookletDuplexPass::Both,
        },
        output_dir: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_illustration_paths() {
        let cfg = create_default_config();
        assert!(cfg.illustrations.cover.source_path.is_none());
        for wd in 0..7u8 {
            let slot = cfg.illustrations.weekday_slot(wd).expect("weekday slot");
            assert!(slot.source_path.is_none());
        }
    }

    #[test]
    fn blank_project_has_no_illustration_paths() {
        let cfg = create_blank_project_config();
        assert!(cfg.illustrations.cover.source_path.is_none());
        assert!(cfg.illustrations.activities.source_path.is_none());
        for wd in 0..7u8 {
            let slot = cfg.illustrations.weekday_slot(wd).expect("weekday slot");
            assert!(slot.source_path.is_none());
        }
    }
}
