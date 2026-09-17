//! Wizard step validation (`validate_wizard_step`).

use crate::calendar::parse_iso_date;
use crate::config::{
    AgendaConfig, MAX_CONTACT_PHONES, contact_row_has_content, validate_contacts,
};
use crate::defaults::WEEKDAY_ORDER;
use crate::paths::resolve_data_dir;
use std::fs;
use std::io;
use std::path::Path;

pub const WIZARD_STEP_COUNT: usize = 5;
const WIZARD_STEP_FILE: &str = "wizard-step";

pub fn is_valid_iso_date(iso: &str) -> bool {
    if iso.len() != 10 || iso.as_bytes()[4] != b'-' || iso.as_bytes()[7] != b'-' {
        return false;
    }
    parse_iso_date(iso).is_ok()
}

/// Human-readable blockers for advancing past this step. Empty = OK.
pub fn validate_wizard_step(step: usize, config: &AgendaConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if step == 0 {
        if config.title.trim().is_empty() {
            errors.push("Indique un titre pour l’agenda.".into());
        }
        if config.school_year_label.trim().is_empty() {
            errors.push("Indique l’année scolaire (ex. 2026-2027).".into());
        }
        if !is_valid_iso_date(&config.rentree) {
            errors.push("Indique une date de rentrée valide.".into());
        }
        if !is_valid_iso_date(&config.fin_des_cours) {
            errors.push("Indique une date de fin des cours valide.".into());
        }
        if is_valid_iso_date(&config.rentree)
            && is_valid_iso_date(&config.fin_des_cours)
            && config.fin_des_cours <= config.rentree
        {
            errors.push("La fin des cours doit être après la rentrée.".into());
        }
        let filled_contacts = config
            .contacts
            .iter()
            .filter(|c| contact_row_has_content(c))
            .count();
        if filled_contacts > MAX_CONTACT_PHONES {
            errors.push(format!(
                "Maximum {MAX_CONTACT_PHONES} contacts."
            ));
        }
        errors.extend(validate_contacts(&config.contacts));
    }

    if step == 1 {
        for (i, h) in config.holidays.iter().enumerate() {
            let label = h.label.trim();
            let ref_label = if label.is_empty() {
                format!("la plage {}", i + 1)
            } else {
                label.to_string()
            };
            if label.is_empty() {
                errors.push(format!("Donne un nom à la plage {}.", i + 1));
            }
            if !is_valid_iso_date(&h.start) || !is_valid_iso_date(&h.end) {
                errors.push(format!(
                    "Complète les dates de début et de fin pour {}.",
                    ref_label
                ));
            } else if h.end < h.start {
                errors.push(format!(
                    "Pour {}, la fin doit être le même jour ou après le début.",
                    ref_label
                ));
            }
        }
    }

    if step == 2 {
        let any_day = WEEKDAY_ORDER.iter().any(|wd| config.school_days.get(*wd));
        if !any_day {
            errors.push("Coche au moins un jour de la semaine.".into());
        }
    }

    errors
}

pub fn can_advance_wizard_step(step: usize, config: &AgendaConfig) -> bool {
    validate_wizard_step(step, config).is_empty()
}

/// GTK / UI: one line per blocker (bulleted list).
pub fn format_wizard_errors(errors: &[String]) -> String {
    errors
        .iter()
        .map(|e| format!("• {}", e))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn load_wizard_step(root: &Path) -> usize {
    let path = resolve_data_dir(root).join(WIZARD_STEP_FILE);
    let raw = fs::read_to_string(&path).unwrap_or_else(|_| "0".into());
    raw.trim()
        .parse::<usize>()
        .ok()
        .filter(|n| *n < WIZARD_STEP_COUNT)
        .unwrap_or(0)
}

pub fn save_wizard_step(root: &Path, step: usize) -> io::Result<()> {
    if step >= WIZARD_STEP_COUNT {
        return Ok(());
    }
    let data = resolve_data_dir(root);
    fs::create_dir_all(&data)?;
    fs::write(data.join(WIZARD_STEP_FILE), format!("{}\n", step))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_default_config;

    #[test]
    fn default_config_passes_all_steps() {
        let config = create_default_config();
        for step in 0..WIZARD_STEP_COUNT {
            assert!(
                can_advance_wizard_step(step, &config),
                "step {}: {:?}",
                step,
                validate_wizard_step(step, &config)
            );
        }
    }

    #[test]
    fn step0_rejects_empty_title() {
        let mut config = create_default_config();
        config.title = "   ".into();
        assert!(!can_advance_wizard_step(0, &config));
    }

    #[test]
    fn step0_allows_up_to_ten_contacts() {
        use crate::config::AgendaContact;
        let mut config = create_default_config();
        config.contacts = (1..=10)
            .map(|i| AgendaContact {
                name: format!("Contact {i}"),
                phone: format!("06 00 00 00 {i:02}"),
            })
            .collect();
        assert!(can_advance_wizard_step(0, &config));
        config.contacts.push(AgendaContact {
            name: "Onze".into(),
            phone: "06 00 00 00 99".into(),
        });
        assert!(!can_advance_wizard_step(0, &config));
    }

    #[test]
    fn step0_rejects_fin_before_rentree() {
        let mut config = create_default_config();
        config.fin_des_cours = config.rentree.clone();
        assert!(!can_advance_wizard_step(0, &config));
    }

    #[test]
    fn format_wizard_errors_joins_lines() {
        let out = format_wizard_errors(&["a".into(), "b".into()]);
        assert!(out.contains("• a"));
        assert!(out.contains("• b"));
    }

    #[test]
    fn step1_rejects_invalid_holiday_dates() {
        let mut config = create_default_config();
        config.holidays[0].end = "not-a-date".into();
        let errs = validate_wizard_step(1, &config);
        assert!(!errs.is_empty());
        assert!(!can_advance_wizard_step(1, &config));
    }

    #[test]
    fn step2_requires_a_weekday() {
        let mut config = create_default_config();
        config.school_days = crate::config::SchoolDaysMap {
            days: [false; 7],
        };
        assert!(!can_advance_wizard_step(2, &config));
    }

    #[test]
    fn wizard_step_roundtrip() {
        let base = std::env::temp_dir().join(format!("marius-wiz-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("assets")).unwrap();
        std::fs::create_dir_all(base.join("fonts")).unwrap();
        std::fs::create_dir_all(base.join("data")).unwrap();
        save_wizard_step(&base, 3).unwrap();
        assert_eq!(load_wizard_step(&base), 3);
        let _ = std::fs::remove_dir_all(&base);
    }
}
