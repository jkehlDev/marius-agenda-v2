//! French national phone numbers (10 digits, no country calling code).

pub const FR_PHONE_DIGIT_COUNT: usize = 10;

pub fn phone_digits(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Display form: `06 12 34 56 78` (groups of two, max 10 digits).
pub fn format_fr_phone_display(digits: &str) -> String {
    let d: String = digits.chars().take(FR_PHONE_DIGIT_COUNT).collect();
    let mut out = String::new();
    for (i, ch) in d.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

/// Sanitize live entry: digits only, capped, spaced for readability.
pub fn format_fr_phone_live(raw: &str) -> String {
    format_fr_phone_display(&phone_digits(raw))
}

pub fn is_valid_fr_national_phone(digits: &str) -> bool {
    digits.len() == FR_PHONE_DIGIT_COUNT && digits.starts_with('0')
}

/// True when the user started a number but it is not a complete national number.
pub fn is_incomplete_fr_phone(raw: &str) -> bool {
    let d = phone_digits(raw);
    !d.is_empty() && !is_valid_fr_national_phone(&d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_groups_ten_digits() {
        assert_eq!(
            format_fr_phone_display("0611223344"),
            "06 11 22 33 44"
        );
    }

    #[test]
    fn live_input_strips_non_digits() {
        assert_eq!(
            format_fr_phone_live("06.12.34.56.78"),
            "06 12 34 56 78"
        );
    }
}
