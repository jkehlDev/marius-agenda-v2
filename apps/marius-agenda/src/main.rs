use agenda_core::PROJECT_EXTENSION;
use agenda_pipeline::{generate_and_export_pdfs, GenerateOptions};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[cfg(any(test, feature = "gtk"))]
mod wizard_nav;

#[cfg(feature = "gtk")]
mod ui;

#[derive(Parser)]
#[command(name = "marius-agenda", about = "Agenda scolaire configurable")]
struct Cli {
    /// Project root (assets, fonts, data/)
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Launch GTK wizard (requires `--features gtk`)
    #[arg(long)]
    gui: bool,

    /// `.marius` project file to open (with `--gui`, or alone to start the GUI)
    #[arg(value_name = "PROJECT", value_hint = clap::ValueHint::FilePath)]
    project: Option<PathBuf>,

    /// Walk through wizard steps under Xvfb (requires `--features gtk`)
    #[arg(long)]
    gui_self_test: bool,

    /// Capture one PNG per wizard step (needs DISPLAY; uses scrot or Gdk texture fallback)
    #[arg(long, value_name = "DIR")]
    gui_screenshot_tour: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate PDFs for selected school periods
    Generate {
        #[arg(long)]
        period: Vec<String>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// List school periods from config
    Periods,
}

fn normalize_gui_project(path: Option<PathBuf>) -> Result<Option<PathBuf>, String> {
    let Some(path) = path.filter(|p| !p.as_os_str().is_empty()) else {
        return Ok(None);
    };
    if path.extension().and_then(|e| e.to_str()) != Some(PROJECT_EXTENSION) {
        return Err(format!(
            "fichier projet attendu : extension .{}",
            PROJECT_EXTENSION
        ));
    }
    Ok(Some(path))
}

fn main() {
    let cli = Cli::parse();
    let root = agenda_core::resolve_app_root(&cli.root);
    let open_project = match normalize_gui_project(cli.project) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Erreur: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(dir) = cli.gui_screenshot_tour {
        #[cfg(feature = "gtk")]
        {
            match ui::run_gui_screenshot_tour(root, dir) {
                Ok(()) => return,
                Err(e) => {
                    eprintln!("GUI screenshot tour FAILED: {}", e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "gtk"))]
        {
            eprintln!("Recompile avec : cargo run -p marius-agenda --features gtk -- --gui-screenshot-tour DIR");
            std::process::exit(1);
        }
    }

    if cli.gui_self_test {
        #[cfg(feature = "gtk")]
        {
            match ui::run_gui_self_test(root) {
                Ok(()) => {
                    println!("GUI self-test OK");
                    return;
                }
                Err(e) => {
                    eprintln!("GUI self-test FAILED: {}", e);
                    std::process::exit(1);
                }
            }
        }
        #[cfg(not(feature = "gtk"))]
        {
            eprintln!("Recompile avec : cargo run -p marius-agenda --features gtk -- --gui-self-test");
            std::process::exit(1);
        }
    }

    let launch_gui = cli.gui || open_project.is_some();
    if launch_gui {
        #[cfg(feature = "gtk")]
        {
            ui::run(root, open_project);
            return;
        }
        #[cfg(not(feature = "gtk"))]
        {
            eprintln!("Recompile avec : cargo run -p marius-agenda --features gtk -- --gui");
            std::process::exit(1);
        }
    }

    match cli.command {
        Some(Commands::Generate { period, output }) => {
            let period_ids = if period.is_empty() { None } else { Some(period) };
            match generate_and_export_pdfs(
                &root,
                GenerateOptions {
                    period_ids,
                    output_dir: output,
                    data_dir: None,
                },
            ) {
                Ok(res) => {
                    println!("PDF → {}", res.pdf_dir.display());
                    for p in &res.pdfs {
                        println!("  ✅ {}", p);
                    }
                }
                Err(err) => {
                    eprintln!("Erreur: {}", err);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Periods) => {
            let config = agenda_core::load_config(&root).expect("config");
            for p in agenda_core::list_school_periods(&config) {
                println!(
                    "{} — {} ({} jours)",
                    p.id,
                    p.label,
                    p.days.len()
                );
            }
        }
        None => {
            eprintln!("Marius Agenda — `generate`, `periods`, ou `--gui` (feature gtk)");
            let config = agenda_core::create_default_config();
            eprintln!(
                "{} période(s) avec la config par défaut {}",
                agenda_core::list_school_periods(&config).len(),
                config.school_year_label
            );
        }
    }
}
