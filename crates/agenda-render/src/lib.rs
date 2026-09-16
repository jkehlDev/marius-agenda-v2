mod assets;
mod context;
mod imposed;
mod pages;
mod util;

pub use assets::{
    activities_image_src, cover_image_src, illustration_data_uri, script_font_face_css,
    weekday_image_src,
};
pub use context::RenderContext;
pub use imposed::render_imposed_html;
pub use pages::{
    build_reading_pages, inject_page_folio, render_period_html, with_page_folios,
};

#[cfg(test)]
mod snapshot_tests;
