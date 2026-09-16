mod webkit;

use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("no HTML files to export")]
    NoHtml,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("webkit print failed: {0}")]
    WebKitFailed(String),
}

pub struct ExportPdfsOptions<'a> {
    pub html_dir: &'a Path,
    pub pdf_dir: &'a Path,
    pub html_files: Option<&'a [String]>,
    pub delete_html_after: bool,
}

fn collect_html_files(html_dir: &Path, html_files: Option<&[String]>) -> Result<Vec<String>, PdfError> {
    let files: Vec<String> = if let Some(list) = html_files {
        list.to_vec()
    } else {
        fs::read_dir(html_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".html"))
            .collect()
    };
    let mut files = files;
    files.sort();
    Ok(files)
}

pub use webkit::{prepare_gtk_runtime, shutdown_pdf_webview};

pub fn export_pdfs(options: ExportPdfsOptions<'_>) -> Result<Vec<String>, PdfError> {
    let files = collect_html_files(options.html_dir, options.html_files)?;
    if files.is_empty() {
        return Err(PdfError::NoHtml);
    }

    webkit::export_html_dir(
        options.html_dir,
        options.pdf_dir,
        &files,
        options.delete_html_after,
    )
}
