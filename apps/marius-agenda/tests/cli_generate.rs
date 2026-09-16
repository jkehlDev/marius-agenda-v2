use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_marius-agenda"))
}

#[test]
fn generate_one_period_writes_pdf() {
    let root = repo_root();
    let out = root.join("target").join("test-output-cli-generate");
    let _ = std::fs::remove_dir_all(&out);

    let status = bin()
        .args([
            "--root",
            root.to_str().expect("utf8"),
            "generate",
            "--period",
            "rentree-premiere-vacances",
            "--output",
            out.to_str().expect("utf8"),
        ])
        .status()
        .expect("run generate");

    assert!(status.success(), "generate failed");

    let pdfs: Vec<_> = std::fs::read_dir(&out)
        .expect("output dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "pdf"))
        .collect();
    assert!(!pdfs.is_empty(), "expected at least one PDF in {}", out.display());
}
