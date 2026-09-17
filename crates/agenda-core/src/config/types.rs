use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `YYYY-MM-DD`
pub type IsoDate = String;

/// JS `Date.getDay()`: 0 = Sunday … 6 = Saturday.
pub type WeekdayIndex = u8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HolidayPeriod {
    pub id: String,
    pub label: String,
    pub start: IsoDate,
    pub end: IsoDate,
}

/// Weekday flags keyed as `"0"` … `"6"` in JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchoolDaysMap {
    pub days: [bool; 7],
}

impl SchoolDaysMap {
    pub fn get(&self, wd: WeekdayIndex) -> bool {
        self.days.get(wd as usize).copied().unwrap_or(false)
    }

    pub fn set(&mut self, wd: WeekdayIndex, value: bool) {
        if let Some(slot) = self.days.get_mut(wd as usize) {
            *slot = value;
        }
    }
}

impl Serialize for SchoolDaysMap {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map: HashMap<String, bool> = self
            .days
            .iter()
            .enumerate()
            .map(|(i, &v)| (i.to_string(), v))
            .collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SchoolDaysMap {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: HashMap<String, bool> = HashMap::deserialize(deserializer)?;
        let mut days = [false; 7];
        for (k, v) in map {
            if let Ok(i) = k.parse::<usize>()
                && i < 7
            {
                days[i] = v;
            }
        }
        Ok(Self { days })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IllustrationSlot {
    pub source_path: Option<String>,
    pub print_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgendaIllustrations {
    pub cover: IllustrationSlot,
    pub activities: IllustrationSlot,
    pub by_weekday: HashMap<String, IllustrationSlot>,
}

impl AgendaIllustrations {
    pub fn weekday_slot(&self, wd: WeekdayIndex) -> Option<&IllustrationSlot> {
        self.by_weekday.get(&wd.to_string())
    }

    pub fn set_weekday_slot(&mut self, wd: WeekdayIndex, slot: IllustrationSlot) {
        self.by_weekday.insert(wd.to_string(), slot);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BookletDuplexPass {
    Both,
    Odd,
    Even,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BookletOptions {
    pub generate_imposed_pdf: bool,
    pub duplex_pass: BookletDuplexPass,
}

use super::contacts::{deserialize_contacts_field, AgendaContact};

/// Contacts printed below the vacations table (last page), max [`MAX_CONTACT_PHONES`].
pub const MAX_CONTACT_PHONES: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgendaConfig {
    pub version: u32,
    pub title: String,
    pub school_year_label: String,
    pub rentree: IsoDate,
    pub fin_des_cours: IsoDate,
    #[serde(
        default,
        rename = "contacts",
        alias = "contactPhones",
        deserialize_with = "deserialize_contacts_field"
    )]
    pub contacts: Vec<AgendaContact>,
    pub holidays: Vec<HolidayPeriod>,
    pub school_days: SchoolDaysMap,
    pub illustrations: AgendaIllustrations,
    pub booklet: BookletOptions,
    pub output_dir: String,
}

pub fn empty_slot() -> IllustrationSlot {
    IllustrationSlot {
        source_path: None,
        print_path: None,
    }
}

pub fn slot(source_path: Option<String>, print_path: Option<String>) -> IllustrationSlot {
    IllustrationSlot {
        source_path,
        print_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_json_without_contacts_defaults_empty() {
        let json = r#"{
            "version": 1,
            "title": "Agenda",
            "schoolYearLabel": "2026-2027",
            "rentree": "2026-09-01",
            "finDesCours": "2027-07-03",
            "holidays": [],
            "schoolDays": {"1": true},
            "illustrations": {
                "cover": {},
                "activities": {},
                "byWeekday": {}
            },
            "booklet": { "generateImposedPdf": false, "duplexPass": "both" },
            "outputDir": ""
        }"#;
        let cfg: AgendaConfig = serde_json::from_str(json).expect("parse");
        assert!(cfg.contacts.is_empty());
    }

    #[test]
    fn config_json_legacy_contact_phones_array() {
        let json = r#"{
            "version": 1,
            "title": "Agenda",
            "schoolYearLabel": "2026-2027",
            "rentree": "2026-09-01",
            "finDesCours": "2027-07-03",
            "contactPhones": ["0611223344", "01 23 45 67 89"],
            "holidays": [],
            "schoolDays": {"1": true},
            "illustrations": {
                "cover": {},
                "activities": {},
                "byWeekday": {}
            },
            "booklet": { "generateImposedPdf": false, "duplexPass": "both" },
            "outputDir": ""
        }"#;
        let cfg: AgendaConfig = serde_json::from_str(json).expect("parse");
        assert_eq!(cfg.contacts.len(), 2);
        assert_eq!(cfg.contacts[0].phone, "06 11 22 33 44");
        assert_eq!(cfg.contacts[1].phone, "01 23 45 67 89");
    }
}
