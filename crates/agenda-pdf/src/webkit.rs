use std::path::Path;
use std::sync::mpsc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, Once};
use std::time::Duration;
use gtk::{PageOrientation, PageSetup, PaperSize, PrintSettings, Unit};
use webkit6::prelude::*;
use webkit6::{LoadEvent, PrintOperation, Settings, WebView};

use crate::PdfError;

/// GtkPrintBackendFile exposes a virtual printer; the name is translated (gettext).
const FILE_PRINTERS: &[&str] = &[
    "Imprimer dans un fichier",
    "Print to File",
    "Print To File",
];

static GTK_INIT: Once = Once::new();

/// WebKit in strict snaps often cannot use DMA-BUF/GBM (NVIDIA, mesa-2404); set before gtk::init.
fn apply_confined_graphics_defaults() {
    if std::env::var_os("SNAP").is_none() {
        return;
    }
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // SAFETY: called once on main thread before gtk::init spawns workers.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
    if std::env::var_os("GSK_RENDERER").is_none() {
        unsafe { std::env::set_var("GSK_RENDERER", "cairo") };
    }
}

/// Call once before GTK UI or WebKit PDF (idempotent).
pub fn prepare_gtk_runtime() {
    GTK_INIT.call_once(|| {
        apply_confined_graphics_defaults();
        gtk::glib::set_application_name("Marius Agenda");
        gtk::glib::set_prgname(Some("marius-agenda"));
        gtk::init().expect("gtk::init for WebKit PDF export");
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_print_backends(Some("file,cups"));
        }
    });
}

fn ensure_gtk() {
    prepare_gtk_runtime();
}

fn printer_not_found_message(msg: &str) -> bool {
    let m = msg.to_lowercase();
    (m.contains("imprimante") && m.contains("trouv"))
        || (m.contains("printer") && m.contains("not found"))
}

fn pump_until(rx: mpsc::Receiver<Result<(), String>>) -> Result<(), PdfError> {
    let ctx = gtk::glib::MainContext::default();
    loop {
        match rx.try_recv() {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(e)) => return Err(PdfError::WebKitFailed(e)),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err(PdfError::WebKitFailed("print operation ended without signal".into()));
            }
            Err(mpsc::TryRecvError::Empty) => {
                while ctx.iteration(false) {}
            }
        }
    }
}

fn drain_gtk_after_export() {
    let ctx = gtk::glib::MainContext::default();
    for _ in 0..40 {
        while ctx.iteration(false) {}
        std::thread::sleep(Duration::from_millis(8));
    }
}

fn idle_webview(engine: &PdfWebView) {
    engine.view.load_uri("about:blank");
    drain_gtk_after_export();
}

thread_local! {
    static PDF_ENGINE: RefCell<Option<PdfWebView>> = const { RefCell::new(None) };
}

fn with_pdf_engine<R>(f: impl FnOnce(&PdfWebView) -> R) -> R {
    PDF_ENGINE.with(|slot| {
        let mut guard = slot.borrow_mut();
        if guard.is_none() {
            *guard = Some(new_pdf_webview());
        }
        f(guard.as_ref().expect("pdf engine"))
    })
}

/// Reset WebKit after export; call before app exit to avoid D-Bus teardown warnings.
pub fn shutdown_pdf_webview() {
    PDF_ENGINE.with(|slot| {
        let mut guard = slot.borrow_mut();
        if let Some(engine) = guard.as_ref() {
            idle_webview(engine);
        }
        *guard = None;
    });
}

#[derive(Clone, Copy)]
enum PrintProfile {
    ImposedA4Landscape,
    PeriodA5Portrait,
}

fn print_profile_for_html(html_path: &Path) -> PrintProfile {
    let name = html_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if name.contains(".livret") {
        PrintProfile::ImposedA4Landscape
    } else {
        PrintProfile::PeriodA5Portrait
    }
}

fn zero_page_margins(setup: &PageSetup) {
    setup.set_top_margin(0.0, Unit::Mm);
    setup.set_bottom_margin(0.0, Unit::Mm);
    setup.set_left_margin(0.0, Unit::Mm);
    setup.set_right_margin(0.0, Unit::Mm);
}

fn apply_print_profile(settings: &PrintSettings, setup: &PageSetup, profile: PrintProfile) {
    zero_page_margins(setup);
    match profile {
        PrintProfile::ImposedA4Landscape => {
            let paper = PaperSize::new(Some(gtk::PAPER_NAME_A4));
            settings.set_orientation(PageOrientation::Landscape);
            setup.set_orientation(PageOrientation::Landscape);
            settings.set_paper_size(&paper);
            setup.set_paper_size(&paper);
        }
        PrintProfile::PeriodA5Portrait => {
            let paper = PaperSize::new(Some(gtk::PAPER_NAME_A5));
            settings.set_orientation(PageOrientation::Portrait);
            setup.set_orientation(PageOrientation::Portrait);
            settings.set_paper_size(&paper);
            setup.set_paper_size(&paper);
        }
    }
}

fn run_print_to_file(
    view: &WebView,
    pdf_uri: &str,
    profile: PrintProfile,
    printer_idx: usize,
    done_tx: &Arc<Mutex<Option<mpsc::Sender<Result<(), String>>>>>,
) {
    if printer_idx >= FILE_PRINTERS.len() {
        if let Some(t) = done_tx.lock().expect("done_tx").take() {
            let _ = t.send(Err("aucune imprimante « fichier » GTK (file print backend)".into()));
        }
        return;
    }

    let settings = PrintSettings::new();
    settings.set_printer(FILE_PRINTERS[printer_idx]);
    settings.set(gtk::PRINT_SETTINGS_OUTPUT_URI, Some(pdf_uri));
    settings.set(gtk::PRINT_SETTINGS_OUTPUT_FILE_FORMAT, Some("pdf"));

    let page_setup = PageSetup::new();
    apply_print_profile(&settings, &page_setup, profile);

    let op = PrintOperation::new(view);
    op.set_print_settings(&settings);
    op.set_page_setup(&page_setup);

    let done_tx_ok = done_tx.clone();
    let done_tx_fail = done_tx.clone();
    let pdf_uri = pdf_uri.to_string();
    let view = view.clone();

    op.connect_finished(move |_| {
        if let Some(t) = done_tx_ok.lock().expect("done_tx").take() {
            let _ = t.send(Ok(()));
        }
    });
    op.connect_failed(move |_, err| {
        let msg = err.to_string();
        if printer_not_found_message(&msg) && printer_idx + 1 < FILE_PRINTERS.len() {
            run_print_to_file(&view, &pdf_uri, profile, printer_idx + 1, &done_tx_fail);
            return;
        }
        if let Some(t) = done_tx_fail.lock().expect("done_tx").take() {
            let _ = t.send(Err(msg));
        }
    });

    op.print();
}

struct ActivePrintJob {
    pdf_uri: String,
    profile: PrintProfile,
    done_tx: Arc<Mutex<Option<mpsc::Sender<Result<(), String>>>>>,
}

#[allow(dead_code)]
pub fn html_file_to_pdf(html_path: &Path, pdf_path: &Path) -> Result<(), PdfError> {
    ensure_gtk();
    with_pdf_engine(|engine| {
        let result = load_and_print_shared(engine, html_path, pdf_path);
        idle_webview(engine);
        result
    })
}

struct PdfWebView {
    view: WebView,
    active: Arc<Mutex<Option<ActivePrintJob>>>,
}

fn new_pdf_webview() -> PdfWebView {
    let view = WebView::new();
    let ws = Settings::default();
    ws.set_print_backgrounds(true);
    view.set_settings(&ws);
    let active: Arc<Mutex<Option<ActivePrintJob>>> = Arc::new(Mutex::new(None));
    view.connect_load_changed({
        let active = active.clone();
        move |view, event| {
            if event != LoadEvent::Finished {
                return;
            }
            let job = active.lock().expect("active job").take();
            if let Some(job) = job {
                run_print_to_file(view, &job.pdf_uri, job.profile, 0, &job.done_tx);
            }
        }
    });
    PdfWebView { view, active }
}

fn load_and_print_shared(engine: &PdfWebView, html_path: &Path, pdf_path: &Path) -> Result<(), PdfError> {
    let html_path = html_path.canonicalize()?;
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file_url = format!("file://{}", html_path.display());
    let pdf_uri = format!("file://{}", pdf_path.display());
    let profile = print_profile_for_html(&html_path);

    let (done_tx, done_rx) = mpsc::channel();
    *engine.active.lock().expect("active job") = Some(ActivePrintJob {
        pdf_uri,
        profile,
        done_tx: Arc::new(Mutex::new(Some(done_tx))),
    });
    engine.view.load_uri(&file_url);
    pump_until(done_rx)
}

pub fn export_html_dir(
    html_dir: &Path,
    pdf_dir: &Path,
    html_files: &[String],
    delete_html_after: bool,
) -> Result<Vec<String>, PdfError> {
    ensure_gtk();
    std::fs::create_dir_all(pdf_dir)?;

    with_pdf_engine(|engine| {
        let mut written = Vec::new();
        for file in html_files {
            let html_path = html_dir.join(file);
            let pdf_name = file.replace(".html", ".pdf").replace(".HTML", ".pdf");
            let pdf_path = pdf_dir.join(&pdf_name);
            load_and_print_shared(engine, &html_path, &pdf_path)?;
            written.push(pdf_name);
            if delete_html_after {
                let _ = std::fs::remove_file(html_path);
            }
        }
        idle_webview(engine);
        Ok(written)
    })
}
