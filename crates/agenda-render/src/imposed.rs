use agenda_core::{
    booklet_side_class, build_imposition, pad_count_to_signature, BookletDuplexPass, SchoolPeriod,
};

use crate::assets::script_font_face_css;
use crate::context::RenderContext;
use crate::pages::{build_reading_pages, inject_page_folio};
use crate::util::escape_html;

const SHARED_CSS: &str = include_str!("../assets/shared.css");
const IMPOSED_CSS: &str = include_str!("../assets/imposed.css");

fn tag_side(page_html: &str, reading_index: usize) -> String {
    let side = booklet_side_class(reading_index);
    page_html.replace(
        "class=\"page page-a5",
        &format!("class=\"page page-a5 page-in-slot {}", side),
    )
}

fn blank_tagged(reading_index: usize) -> String {
    let blank = inject_page_folio(
        r#"<section class="page page-a5 page-blank" aria-hidden="true"></section>"#,
        reading_index + 1,
    );
    tag_side(&blank, reading_index)
}

pub fn render_imposed_html(period: &SchoolPeriod, ctx: &RenderContext) -> String {
    let reading = build_reading_pages(period, ctx);
    let n = pad_count_to_signature(reading.len());
    let mut pages: Vec<String> = Vec::with_capacity(n);
    for i in 0..n {
        if i < reading.len() {
            pages.push(tag_side(&reading[i], i));
        } else {
            pages.push(blank_tagged(i));
        }
    }

    let sheets = build_imposition(reading.len());

    let at = |one_based: usize| -> String {
        pages
            .get(one_based - 1)
            .cloned()
            .unwrap_or_else(|| blank_tagged(one_based - 1))
    };

    let duplex_pass = ctx.config.booklet.duplex_pass;

    let mut fronts: Vec<String> = Vec::new();
    let mut backs: Vec<String> = Vec::new();

    for sheet in &sheets {
        let s = sheet.index;
        let front_left = n - 2 * s;
        let front_right = 2 * s + 1;
        let back_left = 2 * s + 2;
        let back_right = n - 2 * s - 1;

        fronts.push(format!(
            r#"
<section class="sheet" data-sheet="{s}" data-side="front">
  <div class="sheet-slot sheet-slot-left">{}</div>
  <div class="sheet-slot sheet-slot-right">{}</div>
</section>"#,
            at(front_left),
            at(front_right),
        ));
        backs.push(format!(
            r#"
<section class="sheet" data-sheet="{s}" data-side="back">
  <div class="sheet-slot sheet-slot-left">{}</div>
  <div class="sheet-slot sheet-slot-right">{}</div>
</section>"#,
            at(back_left),
            at(back_right),
        ));
    }

    let sheet_html = match duplex_pass {
        BookletDuplexPass::Odd => fronts.join("\n"),
        BookletDuplexPass::Even => backs.into_iter().rev().collect::<Vec<_>>().join("\n"),
        BookletDuplexPass::Both => sheets
            .iter()
            .enumerate()
            .flat_map(|(i, _)| [fronts[i].clone(), backs[i].clone()])
            .collect::<Vec<_>>()
            .join("\n"),
    };

    let font_css = script_font_face_css(ctx.root);

    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
  <meta charset="utf-8" />
  <title>Livret — {}</title>
  <style>{font_css}
{SHARED_CSS}
{IMPOSED_CSS}</style>
</head>
<body>
{sheet_html}
</body>
</html>"#,
        escape_html(&period.label),
    )
}
