use agenda_core::{list_school_periods, period_filename, AgendaConfig, BookletDuplexPass, SchoolPeriod};
use agenda_core::{
    create_default_config, load_config, load_config_from, resolve_output_dir, save_config, save_config_to,
};
use agenda_images::process_config_illustrations;
use agenda_pdf::{export_pdfs, ExportPdfsOptions};

pub use agenda_pdf::{prepare_gtk_runtime, shutdown_pdf_webview};
use agenda_render::{render_imposed_html, render_period_html, RenderContext};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateError {
    #[error("no periods selected")]
    NoPeriods,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Images(#[from] agenda_images::ImageError),
    #[error("{0}")]
    Pdf(#[from] agenda_pdf::PdfError),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub struct GenerateOptions {
    pub period_ids: Option<Vec<String>>,
    pub output_dir: Option<PathBuf>,
    /// Override user data dir (config + print images); defaults to `resolve_data_dir(root)`.
    pub data_dir: Option<PathBuf>,
}

pub struct GenerateResult {
    pub pdfs: Vec<String>,
    pub pdf_dir: PathBuf,
}

/// HTML on disk in a temp dir; call `export_prepared_pdf_generation` on the GTK main thread when using WebKit.
pub struct PreparedPdfGeneration {
    _tmpdir: TempDir,
    html_dir: PathBuf,
    html_files: Vec<String>,
    pdf_dir: PathBuf,
}

/// Basename of the PDF that would be written for this period (GUI overwrite check).
pub fn period_pdf_filename(period: &SchoolPeriod, config: &AgendaConfig) -> String {
    let base = period_filename(period);
    if config.booklet.generate_imposed_pdf {
        let suffix = match config.booklet.duplex_pass {
            BookletDuplexPass::Odd => ".livret-impair",
            BookletDuplexPass::Even => ".livret-pair",
            BookletDuplexPass::Both => ".livret",
        };
        format!("{}{}.pdf", base, suffix)
    } else {
        format!("{}.pdf", base)
    }
}

/// Existing PDF paths under `pdf_dir` that would be replaced for the given period ids.
pub fn existing_pdf_overwrites(
    config: &AgendaConfig,
    pdf_dir: &Path,
    period_ids: &[String],
) -> Vec<PathBuf> {
    list_school_periods(config)
        .into_iter()
        .filter(|p| period_ids.contains(&p.id))
        .map(|p| pdf_dir.join(period_pdf_filename(&p, config)))
        .filter(|path| path.is_file())
        .collect()
}

fn render_period(period: &SchoolPeriod, root: &Path, config: &AgendaConfig) -> (String, String) {
    let ctx = RenderContext {
        root,
        config,
        with_art: true,
    };
    let base = period_filename(period);
    if config.booklet.generate_imposed_pdf {
        let pass = &config.booklet.duplex_pass;
        let suffix = match pass {
            BookletDuplexPass::Odd => ".livret-impair.html",
            BookletDuplexPass::Even => ".livret-pair.html",
            BookletDuplexPass::Both => ".livret.html",
        };
        let name = format!("{}{}", base, suffix);
        let html = render_imposed_html(period, &ctx);
        (name, html)
    } else {
        let name = format!("{}.html", base);
        let html = render_period_html(period, &ctx);
        (name, html)
    }
}

fn load_config_for(root: &Path, data_dir: Option<&Path>) -> Result<AgendaConfig, GenerateError> {
    if let Some(dir) = data_dir {
        let file = agenda_core::config_path(dir);
        if file.exists() {
            return Ok(load_config_from(dir)?);
        }
        let defaults = create_default_config();
        save_config_to(root, dir, &defaults)?;
        return Ok(defaults);
    }
    Ok(load_config(root)?)
}

fn save_config_for(root: &Path, data_dir: Option<&Path>, config: &AgendaConfig) -> Result<(), GenerateError> {
    if let Some(dir) = data_dir {
        save_config_to(root, dir, config)?;
    } else {
        save_config(root, config)?;
    }
    Ok(())
}

pub fn prepare_pdf_generation(root: &Path, options: GenerateOptions) -> Result<PreparedPdfGeneration, GenerateError> {
    let data_dir = options.data_dir.as_deref();
    let mut config = load_config_for(root, data_dir)?;
    config = process_config_illustrations(root, &config, data_dir)?;
    save_config_for(root, data_dir, &config)?;

    let all = list_school_periods(&config);
    let periods: Vec<SchoolPeriod> = match &options.period_ids {
        Some(ids) if !ids.is_empty() => all.into_iter().filter(|p| ids.contains(&p.id)).collect(),
        _ => all,
    };

    if periods.is_empty() {
        return Err(GenerateError::NoPeriods);
    }

    let pdf_dir = resolve_output_dir(
        root,
        options
            .output_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .as_deref()
            .unwrap_or(&config.output_dir),
    );
    fs::create_dir_all(&pdf_dir)?;

    let tmp = TempDir::new()?;
    let html_dir = tmp.path().to_path_buf();
    let mut html_files = Vec::new();

    for period in &periods {
        let (name, html) = render_period(period, root, &config);
        fs::write(html_dir.join(&name), html)?;
        html_files.push(name);
    }

    Ok(PreparedPdfGeneration {
        _tmpdir: tmp,
        html_dir,
        html_files,
        pdf_dir,
    })
}

pub fn export_prepared_pdf_generation(
    prepared: PreparedPdfGeneration,
) -> Result<GenerateResult, GenerateError> {
    let pdfs = export_pdfs(ExportPdfsOptions {
        html_dir: &prepared.html_dir,
        pdf_dir: &prepared.pdf_dir,
        html_files: Some(&prepared.html_files),
        delete_html_after: true,
    })?;

    Ok(GenerateResult {
        pdfs,
        pdf_dir: prepared.pdf_dir,
    })
}

pub fn generate_and_export_pdfs(root: &Path, options: GenerateOptions) -> Result<GenerateResult, GenerateError> {
    let prepared = prepare_pdf_generation(root, options)?;
    export_prepared_pdf_generation(prepared)
}
