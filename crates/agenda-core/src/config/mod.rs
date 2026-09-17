mod contacts;
mod phone;
mod types;

pub use contacts::{
    AgendaContact, contact_row_has_content, empty_contact, normalize_contact, normalized_contacts,
    validate_contacts,
};
pub use phone::{
    format_fr_phone_display, format_fr_phone_live, is_incomplete_fr_phone, is_valid_fr_national_phone,
    phone_digits,
};
pub use types::*;
