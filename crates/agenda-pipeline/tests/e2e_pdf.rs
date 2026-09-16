//! End-to-end: config → images → HTML → PDF (WebKitGTK; needs DISPLAY or xvfb).
//! Run alone if the process aborts during WebKit teardown in parallel CI: `cargo test -p agenda-pipeline --test e2e_pdf -- --test-threads=1`

use agenda_core::{create_default_config, list_school_periods, save_config_to};
use agenda_pipeline::{generate_and_export_pdfs, GenerateOptions};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn assert_pdf_file(path: &PathBuf) {
    let bytes = fs::read(path).expect("read pdf");
    assert!(bytes.starts_with(b"%PDF"), "not a PDF: {}", path.display());
    assert!(
        bytes.len() > 50_000,
        "PDF suspiciously small: {} ({} bytes)",
        path.display(),
        bytes.len()
    );
}

#[test]
fn e2e_pdf_pipeline_one_and_all_periods() {
    let root = repo_root();
    let tmp_data = tempfile::tempdir().expect("tempdir");
    let config = create_default_config();
    save_config_to(&root, tmp_data.path(), &config).expect("save config");

    let period = list_school_periods(&config)[0].clone();
    let out_one = tempfile::tempdir().expect("out dir");

    let one = generate_and_export_pdfs(
        &root,
        GenerateOptions {
            period_ids: Some(vec![period.id.clone()]),
            output_dir: Some(out_one.path().to_path_buf()),
            data_dir: Some(tmp_data.path().to_path_buf()),
        },
    )
    .expect("generate one period");

    assert_eq!(one.pdfs.len(), 1);
    assert_pdf_file(&one.pdf_dir.join(&one.pdfs[0]));

    let out_all = tempfile::tempdir().expect("out dir all");
    let all = generate_and_export_pdfs(
        &root,
        GenerateOptions {
            period_ids: None,
            output_dir: Some(out_all.path().to_path_buf()),
            data_dir: Some(tmp_data.path().to_path_buf()),
        },
    )
    .expect("generate all periods");

    assert_eq!(all.pdfs.len(), 5);
    for name in &all.pdfs {
        assert_pdf_file(&all.pdf_dir.join(name));
    }
}
