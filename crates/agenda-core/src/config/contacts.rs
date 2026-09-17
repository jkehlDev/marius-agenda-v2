use super::phone::{format_fr_phone_display, is_valid_fr_national_phone, phone_digits};
use super::MAX_CONTACT_PHONES;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgendaContact {
    pub name: String,
    pub phone: String,
}

pub fn empty_contact() -> AgendaContact {
    AgendaContact::default()
}

pub fn contact_row_has_content(c: &AgendaContact) -> bool {
    !c.name.trim().is_empty() || !phone_digits(&c.phone).is_empty()
}

pub fn normalize_contact(c: &AgendaContact) -> Option<AgendaContact> {
    let name = c.name.trim().to_string();
    let digits = phone_digits(&c.phone);
    if name.is_empty() || !is_valid_fr_national_phone(&digits) {
        return None;
    }
    Some(AgendaContact {
        name,
        phone: format_fr_phone_display(&digits),
    })
}

/// Contacts to print (trimmed, formatted phones), max [`MAX_CONTACT_PHONES`].
pub fn normalized_contacts(entries: &[AgendaContact]) -> Vec<AgendaContact> {
    entries
        .iter()
        .filter_map(normalize_contact)
        .take(MAX_CONTACT_PHONES)
        .collect()
}

pub fn validate_contacts(entries: &[AgendaContact]) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, c) in entries.iter().enumerate() {
        if !contact_row_has_content(c) {
            continue;
        }
        let name = c.name.trim();
        let digits = phone_digits(&c.phone);
        if name.is_empty() {
            errors.push(format!(
                "Contact {} : indique un nom.",
                i + 1
            ));
        }
        if digits.is_empty() {
            errors.push(format!(
                "Contact {} : indique un numéro.",
                i + 1
            ));
        } else if !is_valid_fr_national_phone(&digits) {
            errors.push(format!(
                "Contact {} : numéro invalide (10 chiffres commençant par 0, sans indicatif, ex. 06 12 34 56 78).",
                i + 1
            ));
        }
    }
    errors
}

pub fn deserialize_contacts_field<'de, D>(deserializer: D) -> Result<Vec<AgendaContact>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ContactsJson {
        Structured(Vec<AgendaContact>),
        LegacyPhones(Vec<String>),
    }

    let raw = Option::<ContactsJson>::deserialize(deserializer)?;
    Ok(match raw {
        None => Vec::new(),
        Some(ContactsJson::Structured(list)) => list,
        Some(ContactsJson::LegacyPhones(phones)) => phones
            .into_iter()
            .map(|p| AgendaContact {
                name: String::new(),
                phone: format_fr_phone_display(&phone_digits(&p)),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_contacts_skips_empty_rows() {
        let raw = vec![
            AgendaContact {
                name: " Secrétariat ".into(),
                phone: "06 11 22 33 44".into(),
            },
            AgendaContact::default(),
            AgendaContact {
                name: "".into(),
                phone: "0712345678".into(),
            },
        ];
        let out = normalized_contacts(&raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Secrétariat");
    }

    #[test]
    fn validate_requires_name_and_complete_phone() {
        let entries = vec![
            AgendaContact {
                name: "".into(),
                phone: "06 12 34 56 78".into(),
            },
            AgendaContact {
                name: "Test".into(),
                phone: "06 12".into(),
            },
        ];
        let errs = validate_contacts(&entries);
        assert_eq!(errs.len(), 2);
    }

}
