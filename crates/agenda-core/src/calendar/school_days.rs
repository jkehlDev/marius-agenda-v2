use crate::config::{AgendaConfig, HolidayPeriod, IsoDate, WeekdayIndex};
use crate::defaults::WEEKDAY_ORDER;
use time::{Date, Duration};

#[derive(Debug, Clone)]
pub struct SchoolDay {
    pub date: Date,
    pub iso: IsoDate,
    pub weekday: WeekdayIndex,
    pub day_of_month: u8,
    pub month_index: u8,
    pub year: i32,
}

#[derive(Debug, Clone)]
pub struct SchoolPeriod {
    pub id: String,
    pub slug: String,
    pub label: String,
    pub start: IsoDate,
    pub end: IsoDate,
    pub days: Vec<SchoolDay>,
}

const MONTHS_FR: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

const WEEKDAYS_FR: [&str; 7] = [
    "dimanche",
    "lundi",
    "mardi",
    "mercredi",
    "jeudi",
    "vendredi",
    "samedi",
];

pub fn parse_iso_date(iso: &str) -> Result<Date, time::error::Parse> {
    Date::parse(iso, &time::format_description::well_known::Iso8601::DATE)
}

pub fn to_iso_date(date: Date) -> IsoDate {
    date.format(&time::format_description::well_known::Iso8601::DATE)
        .unwrap_or_default()
}

fn add_days(date: Date, days: i64) -> Date {
    date + Duration::days(days)
}

pub fn day_before(iso: &str) -> IsoDate {
    let d = parse_iso_date(iso).unwrap_or_else(|_| Date::from_calendar_date(1970, time::Month::January, 1).unwrap());
    to_iso_date(add_days(d, -1))
}

pub fn day_after(iso: &str) -> IsoDate {
    let d = parse_iso_date(iso).unwrap_or_else(|_| Date::from_calendar_date(1970, time::Month::January, 1).unwrap());
    to_iso_date(add_days(d, 1))
}

pub fn month_name_fr(month_index: u8) -> &'static str {
    MONTHS_FR.get(month_index as usize).copied().unwrap_or("")
}

pub fn weekday_name_fr(weekday: WeekdayIndex) -> &'static str {
    WEEKDAYS_FR.get(weekday as usize).copied().unwrap_or("")
}

pub fn active_weekdays(config: &AgendaConfig) -> Vec<WeekdayIndex> {
    WEEKDAY_ORDER
        .iter()
        .copied()
        .filter(|wd| config.school_days.get(*wd))
        .collect()
}

fn weekday_index(date: Date) -> WeekdayIndex {
    date.weekday().number_days_from_sunday() as WeekdayIndex
}

fn is_agenda_weekday(weekday: WeekdayIndex, config: &AgendaConfig) -> bool {
    config.school_days.get(weekday)
}

fn is_inside_holiday(date: Date, holidays: &[HolidayPeriod]) -> bool {
    holidays.iter().any(|h| {
        let start = parse_iso_date(&h.start).ok();
        let end = parse_iso_date(&h.end).ok();
        match (start, end) {
            (Some(s), Some(e)) => date >= s && date <= e,
            _ => false,
        }
    })
}

fn to_school_day(date: Date) -> SchoolDay {
    SchoolDay {
        date,
        iso: to_iso_date(date),
        weekday: weekday_index(date),
        day_of_month: date.day(),
        month_index: date.month() as u8 - 1,
        year: date.year(),
    }
}

pub fn list_agenda_days(config: &AgendaConfig) -> Vec<SchoolDay> {
    let start = parse_iso_date(&config.rentree).unwrap_or_else(|_| {
        Date::from_calendar_date(2026, time::Month::September, 1).unwrap()
    });
    let end = parse_iso_date(&config.fin_des_cours).unwrap_or_else(|_| {
        Date::from_calendar_date(2027, time::Month::July, 3).unwrap()
    });

    let mut days = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        let wd = weekday_index(cursor);
        if is_agenda_weekday(wd, config) && !is_inside_holiday(cursor, &config.holidays) {
            days.push(to_school_day(cursor));
        }
        cursor = add_days(cursor, 1);
    }
    days
}

fn period_meta_after(holiday_id: &str, index: usize) -> (String, String, String) {
    let padded = format!("{:02}", index + 2);
    match holiday_id {
        "toussaint" => (
            "apres-toussaint-noel".into(),
            "02-apres-toussaint-noel".into(),
            "Après la Toussaint jusqu'aux vacances de Noël".into(),
        ),
        "noel" => (
            "apres-noel-hiver".into(),
            "03-apres-noel-hiver".into(),
            "Après Noël jusqu'aux vacances d'hiver".into(),
        ),
        "hiver" => (
            "apres-hiver-printemps".into(),
            "04-apres-hiver-printemps".into(),
            "Après l'hiver jusqu'aux vacances de printemps".into(),
        ),
        "printemps" => (
            "apres-printemps-ete".into(),
            "05-apres-printemps-ete".into(),
            "Après le printemps jusqu'aux vacances d'été".into(),
        ),
        _ => (
            format!("period-{}", padded),
            format!("{}-period", padded),
            format!("Période {}", index + 2),
        ),
    }
}

pub fn list_school_periods(config: &AgendaConfig) -> Vec<SchoolPeriod> {
    let all_days = list_agenda_days(config);
    let holidays = &config.holidays;

    struct Boundary {
        id: String,
        slug: String,
        label: String,
        start: IsoDate,
        end: IsoDate,
    }

    let mut boundaries: Vec<Boundary> = Vec::new();

    if holidays.is_empty() {
        boundaries.push(Boundary {
            id: "annee-complete".into(),
            slug: "01-annee-complete".into(),
            label: format!("Année scolaire {}", config.school_year_label),
            start: config.rentree.clone(),
            end: day_before(&config.fin_des_cours),
        });
    } else {
        let first = &holidays[0];
        let label = if first.id == "toussaint" {
            "De la rentrée aux vacances de la Toussaint".into()
        } else {
            format!("De la rentrée aux vacances ({})", first.label)
        };
        boundaries.push(Boundary {
            id: "rentree-premiere-vacances".into(),
            slug: "01-rentree-toussaint".into(),
            label,
            start: config.rentree.clone(),
            end: day_before(&first.start),
        });

        for (i, current) in holidays.iter().enumerate() {
            let period_end = if let Some(next) = holidays.get(i + 1) {
                day_before(&next.start)
            } else {
                day_before(&config.fin_des_cours)
            };
            let (id, slug, label) = period_meta_after(&current.id, i);
            boundaries.push(Boundary {
                id,
                slug,
                label,
                start: day_after(&current.end),
                end: period_end,
            });
        }
    }

    boundaries
        .into_iter()
        .filter_map(|b| {
            let start = parse_iso_date(&b.start).ok();
            let end = parse_iso_date(&b.end).ok();
            let days: Vec<SchoolDay> = match (start, end) {
                (Some(s), Some(e)) => all_days
                    .iter()
                    .filter(|d| d.date >= s && d.date <= e)
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            };
            if days.is_empty() {
                None
            } else {
                Some(SchoolPeriod {
                    id: b.id,
                    slug: b.slug,
                    label: b.label,
                    start: b.start,
                    end: b.end,
                    days,
                })
            }
        })
        .collect()
}

pub fn period_filename(period: &SchoolPeriod) -> String {
    let first = period.days.first().map(|d| d.iso.as_str()).unwrap_or("");
    let last = period.days.last().map(|d| d.iso.as_str()).unwrap_or("");
    format!("{}_{}_au_{}", period.slug, first, last)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgendaConfig;
    use crate::create_default_config;

    #[test]
    fn default_config_yields_five_periods() {
        let config = create_default_config();
        let periods = list_school_periods(&config);
        assert_eq!(periods.len(), 5);
        assert!(periods[0].days.len() > 10);
    }

    #[test]
    fn default_config_json_roundtrip() {
        let config = create_default_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let back: AgendaConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.school_year_label, "2026-2027");
        assert_eq!(back.holidays.len(), 4);
    }
}
