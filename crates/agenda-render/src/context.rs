use agenda_core::AgendaConfig;
use std::path::Path;

pub struct RenderContext<'a> {
    pub root: &'a Path,
    pub config: &'a AgendaConfig,
    pub with_art: bool,
}
