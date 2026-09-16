#[cfg(test)]
mod tests {
    use agenda_core::{create_default_config, list_school_periods};
    use std::path::PathBuf;

    use crate::{render_imposed_html, render_period_html, RenderContext};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    #[test]
    fn renders_first_period_html() {
        let root = repo_root();
        let config = create_default_config();
        let period = list_school_periods(&config)[0].clone();
        let ctx = RenderContext {
            root: &root,
            config: &config,
            with_art: true,
        };
        let html = render_imposed_html(&period, &ctx);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("sheet-slot"));
        assert!(html.contains(&period.label));
        assert!(
            html.len() > 30_000,
            "imposed HTML should embed script font (got {} bytes)",
            html.len()
        );
        assert!(html.contains("@font-face"));
    }

    #[test]
    fn renders_reading_order_html() {
        let root = repo_root();
        let config = create_default_config();
        let mut cfg = config.clone();
        cfg.booklet.generate_imposed_pdf = false;
        let period = list_school_periods(&config)[0].clone();
        let ctx = RenderContext {
            root: &root,
            config: &cfg,
            with_art: true,
        };
        let html = render_period_html(&period, &ctx);
        assert!(html.contains("page-day"));
        assert!(!html.contains("sheet-slot"));
    }
}
