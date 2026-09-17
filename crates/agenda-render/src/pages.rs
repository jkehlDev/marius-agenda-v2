use agenda_core::{
    active_weekdays, booklet_side_class, month_name_fr, parse_iso_date, weekday_name_fr, AgendaConfig,
    SchoolDay, SchoolPeriod, WeekdayIndex, WEEKDAY_LABELS_FR,
};

use crate::assets::{activities_image_src, cover_image_src, script_font_face_css, weekday_image_src};
use crate::context::RenderContext;
use crate::util::{capitalize, escape_html};

const SHARED_CSS: &str = include_str!("../assets/shared.css");

fn weekday_label(wd: WeekdayIndex) -> &'static str {
    WEEKDAY_LABELS_FR
        .iter()
        .find(|(id, _)| *id == wd)
        .map(|(_, l)| *l)
        .unwrap_or("")
}

fn week_frieze(today: WeekdayIndex, school_days: &AgendaConfig) -> String {
    let order: [WeekdayIndex; 7] = [1, 2, 3, 4, 5, 6, 0];
    let cells = order
        .iter()
        .map(|wd| {
            let is_today = *wd == today;
            let is_off = !school_days.school_days.get(*wd);
            let classes = [
                "cell",
                if is_today { "is-today" } else { "" },
                if is_off && !is_today { "is-off" } else { "" },
            ]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
            format!(
                r#"<span class="{}"><span class="cell-label">{}</span></span>"#,
                classes,
                capitalize(weekday_name_fr(*wd))
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<div class="week-frieze" aria-label="Frise de la semaine">{cells}</div>"#
    )
}

fn cover_page(
    period: &SchoolPeriod,
    config: &AgendaConfig,
    cover_src: Option<&str>,
    with_art: bool,
) -> String {
    let first = &period.days[0];
    let last = period.days.last().expect("period has days");
    let range = format!(
        "{} {} {} → {} {} {}",
        first.day_of_month,
        month_name_fr(first.month_index),
        first.year,
        last.day_of_month,
        month_name_fr(last.month_index),
        last.year
    );
    let art = if with_art {
        cover_src.map(|src| {
            format!(
                r#"<div class="cover-coloring"><img class="cover-coloring-img" src="{}" alt="" /></div>"#,
                src
            )
        }).unwrap_or_default()
    } else {
        String::new()
    };
    let cover_class = if with_art && cover_src.is_some() {
        " cover-with-art"
    } else {
        ""
    };

    format!(
        r#"<section class="page page-a5 cover{cover_class}">
    <div class="cover-top">
      <h1 class="cover-title">{}</h1>
      <p class="cover-subtitle">Année scolaire {}</p>
      <p class="cover-period">{}<br>{}</p>
    </div>
    {art}
    <div class="cover-fields">
      <div class="field"><label>Prénom</label><div class="write-line"></div></div>
      <div class="field"><label>Nom</label><div class="write-line"></div></div>
    </div>
  </section>"#,
        escape_html(&config.title),
        escape_html(&config.school_year_label),
        escape_html(&period.label),
        escape_html(&range),
    )
}

#[derive(Clone)]
struct ActivityCol {
    label: &'static str,
    wed: bool,
}

fn activities_pages(
    config: &AgendaConfig,
    activities_src: Option<&str>,
    with_art: bool,
) -> Vec<String> {
    let days: Vec<ActivityCol> = active_weekdays(config)
        .iter()
        .map(|wd| ActivityCol {
            label: weekday_label(*wd),
            wed: *wd == 3,
        })
        .collect();

    let day_count = days.len();
    let first_count = day_count.min(3);
    let mut chunks: Vec<Vec<ActivityCol>> = if day_count <= 3 {
        vec![days]
    } else {
        vec![days[..first_count].to_vec(), days[first_count..].to_vec()]
    };

    if with_art && activities_src.is_some() && chunks.len() == 1 && day_count > 0 {
        chunks.push(vec![]);
    }

    chunks
        .iter()
        .enumerate()
        .map(|(page_index, cols)| {
            let show_art = with_art && activities_src.is_some() && page_index == chunks.len() - 1;
            let art = if show_art {
                format!(
                    r#"<div class="activities-art"><img class="activities-art-img" src="{}" alt="" /></div>"#,
                    activities_src.unwrap()
                )
            } else {
                String::new()
            };
            let art_class = if show_art { " activities-with-art" } else { "" };
            let grid = if cols.is_empty() {
                String::new()
            } else {
                let cols_html = cols
                    .iter()
                    .map(|c| {
                        let wed = if c.wed { " is-wed" } else { "" };
                        format!(
                            r#"<div class="activity-col">
        <div class="col-title{wed}">{}</div>
        <div class="seyes seyes-tall"></div>
      </div>"#,
                            c.label
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("");
                format!(r#"<div class="activities-grid">{cols_html}</div>"#)
            };

            format!(
                r#"<section class="page page-a5 activities{art_class}">
    <header class="activities-header">
      <h1>Mes activités de la semaine</h1>
    </header>
    {grid}
    {art}
  </section>"#
            )
        })
        .collect()
}

fn day_page(day: &SchoolDay, config: &AgendaConfig, mascot_src: Option<&str>) -> String {
    let month_num = format!("{:02}", day.month_index + 1);
    let mascot = mascot_src
        .map(|src| format!(r#"<img class="day-draw-mascot" src="{}" alt="" />"#, src))
        .unwrap_or_default();

    format!(
        r#"<section class="page page-a5 page-day">
    <header class="day-header">
      <div class="day-header-top">
        <div class="day-meta">
          <div class="weekday">{}</div>
          <div class="month-line">{} ({}) · {}</div>
        </div>
        <div class="day-number" aria-label="Jour du mois">{}</div>
      </div>
      {}
    </header>
    <div class="day-body">
      <div class="homework-label">Devoirs</div>
      <div class="seyes"></div>
    </div>
    <div class="day-draw">{mascot}</div>
    <div class="day-corner" aria-hidden="true" title="À plier ou déchirer">
      <span class="day-corner-fold"></span>
      <span class="day-corner-scissors">
        <svg viewBox="0 0 24 24" width="100%" height="100%" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="6" cy="6" r="2.4" />
          <circle cx="6" cy="18" r="2.4" />
          <path d="M8.2 7.6 20 18.5" />
          <path d="M8.2 16.4 20 5.5" />
          <path d="M14.2 12h0.01" />
        </svg>
      </span>
    </div>
  </section>"#,
        capitalize(weekday_name_fr(day.weekday)),
        month_name_fr(day.month_index),
        month_num,
        day.year,
        day.day_of_month,
        week_frieze(day.weekday, config),
    )
}

fn with_booklet_sides(pages: Vec<String>) -> Vec<String> {
    pages
        .into_iter()
        .enumerate()
        .map(|(index, html)| {
            let side = booklet_side_class(index);
            html.replace(
                "class=\"page page-a5",
                &format!("class=\"page page-a5 {}", side),
            )
        })
        .collect()
}

pub fn inject_page_folio(html: &str, page_number: usize) -> String {
    let open_end = html.find('>');
    let open_tag = if let Some(end) = open_end {
        &html[..end]
    } else {
        ""
    };
    if open_tag.contains("cover") {
        return html.to_string();
    }
    let folio = format!(
        r#"<div class="page-folio" aria-hidden="true">- {} -</div>"#,
        page_number
    );
    if let Some(pos) = html.rfind("</section>") {
        let mut out = html.to_string();
        out.insert_str(pos, &folio);
        return out;
    }
    html.to_string()
}

pub fn with_page_folios(pages: Vec<String>) -> Vec<String> {
    pages
        .into_iter()
        .enumerate()
        .map(|(index, html)| inject_page_folio(&html, index + 1))
        .collect()
}

fn format_holiday_range(start_iso: &str, end_iso: &str) -> String {
    let fmt = |iso: &str| {
        let d = parse_iso_date(iso).ok();
        d.map(|date| format!("{:02}/{:02}/{}", date.day(), date.month() as u8, date.year()))
            .unwrap_or_default()
    };
    format!("du {} au {}", fmt(start_iso), fmt(end_iso))
}

fn fmt_long(iso: &str) -> String {
    let d = parse_iso_date(iso).ok();
    d.map(|date| {
        let wd = date.weekday().number_days_from_sunday() as WeekdayIndex;
        format!(
            "{} {:02}/{:02}/{}",
            weekday_name_fr(wd),
            date.day(),
            date.month() as u8,
            date.year()
        )
    })
    .unwrap_or_default()
}

fn vacations_contacts_html(config: &AgendaConfig) -> String {
    use agenda_core::normalized_contacts;
    let contacts = normalized_contacts(&config.contacts);
    if contacts.is_empty() {
        return String::new();
    }
    let items = contacts
        .iter()
        .map(|c| {
            let line = if c.name.is_empty() {
                escape_html(&c.phone)
            } else if c.phone.is_empty() {
                escape_html(&c.name)
            } else {
                format!(
                    "{} — {}",
                    escape_html(&c.name),
                    escape_html(&c.phone)
                )
            };
            format!("<li>{line}</li>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<div class="vacations-contacts">
      <h2>Contacts</h2>
      <ul class="vacations-contacts-list">{items}</ul>
    </div>"#
    )
}

fn vacations_page(config: &AgendaConfig) -> String {
    let rows = config
        .holidays
        .iter()
        .map(|h| {
            format!(
                r#"<tr>
      <td>{}</td>
      <td>{}</td>
    </tr>"#,
                escape_html(&h.label),
                format_holiday_range(&h.start, &h.end)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let contacts = vacations_contacts_html(config);

    format!(
        r#"<section class="page page-a5 vacations">
    <h1>Calendrier des vacances</h1>
    <p class="zone">Année scolaire {}</p>
    <table>
      <thead><tr><th>Période</th><th>Dates</th></tr></thead>
      <tbody>
        <tr><td>Rentrée scolaire</td><td>{}</td></tr>
        {rows}
        <tr><td>Fin des cours</td><td>{}</td></tr>
      </tbody>
    </table>
    {contacts}
  </section>"#,
        escape_html(&config.school_year_label),
        fmt_long(&config.rentree),
        fmt_long(&config.fin_des_cours),
    )
}

pub fn build_reading_pages(period: &SchoolPeriod, ctx: &RenderContext) -> Vec<String> {
    let cover_src = if ctx.with_art {
        cover_image_src(ctx.root, ctx.config)
    } else {
        None
    };
    let activities_src = if ctx.with_art {
        activities_image_src(ctx.root, ctx.config)
    } else {
        None
    };

    let mut pages = vec![cover_page(
        period,
        ctx.config,
        cover_src.as_deref(),
        ctx.with_art,
    )];
    pages.extend(activities_pages(
        ctx.config,
        activities_src.as_deref(),
        ctx.with_art,
    ));
    for day in &period.days {
        let mascot = if ctx.with_art {
            weekday_image_src(ctx.root, ctx.config, day.weekday)
        } else {
            None
        };
        pages.push(day_page(day, ctx.config, mascot.as_deref()));
    }
    pages.push(vacations_page(ctx.config));

    with_page_folios(pages)
}

pub fn render_period_html(period: &SchoolPeriod, ctx: &RenderContext) -> String {
    let pages = with_booklet_sides(build_reading_pages(period, ctx));
    let body = pages.join("\n");
    let font_css = script_font_face_css(ctx.root);

    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Agenda — {}</title>
  <style>{font_css}
{SHARED_CSS}</style>
</head>
<body>
{body}
</body>
</html>"#,
        escape_html(&period.label),
    )
}
