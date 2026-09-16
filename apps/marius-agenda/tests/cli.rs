use agenda_core::{create_default_config, save_config};
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

fn isolated_app_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!("marius-cli-root-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let repo = repo_root();
    std::fs::create_dir_all(root.join("data")).unwrap();
    for sub in ["assets", "fonts"] {
        std::fs::create_dir_all(root.join(sub)).unwrap();
        for ent in std::fs::read_dir(repo.join(sub)).expect("read share dir") {
            let ent = ent.unwrap();
            let dest = root.join(sub).join(ent.file_name());
            if ent.file_type().unwrap().is_dir() {
                copy_dir_all(&ent.path(), &dest).unwrap();
            } else {
                std::fs::copy(ent.path(), dest).unwrap();
            }
        }
    }
    save_config(&root, &create_default_config()).unwrap();
    root
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for ent in std::fs::read_dir(src)? {
        let ent = ent?;
        let to = dst.join(ent.file_name());
        if ent.file_type()?.is_dir() {
            copy_dir_all(&ent.path(), &to)?;
        } else {
            std::fs::copy(ent.path(), to)?;
        }
    }
    Ok(())
}

#[test]
fn cli_periods_lists_five() {
    let root = isolated_app_root();
    let out = bin()
        .args(["--root", root.to_str().expect("utf8 root"), "periods"])
        .output()
        .expect("run periods");
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("rentree-premiere-vacances"));
    assert_eq!(stdout.lines().filter(|l| l.contains("—")).count(), 5);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn gui_rejects_non_marius_project_argument() {
    let out = bin()
        .args(["--gui", "/tmp/not-a-project.json"])
        .output()
        .expect("run");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(".marius"));
}
