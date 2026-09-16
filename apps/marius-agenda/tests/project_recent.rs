//! Recent-projects JSON behaviour (isolated path via env).

use agenda_core::{create_new_project, save_project_archive};
use std::fs;
use std::path::PathBuf;

fn with_recent_file<F: FnOnce(&PathBuf)>(f: F) {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let json = std::env::temp_dir().join(format!(
        "marius-recent-it-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_file(&json);
    unsafe { std::env::set_var("MARIUS_AGENDA_RECENT_PROJECTS", json.to_string_lossy().as_ref()) };
    f(&json);
    let _ = fs::remove_file(&json);
    unsafe { std::env::remove_var("MARIUS_AGENDA_RECENT_PROJECTS") };
}

// Prune+persist covered in `agenda-core::project::load_recent_persists_prune_to_json_file`.
// Env var is process-global — keep one integration test here to avoid parallel test races.

#[test]
fn save_does_not_touch_recent_when_skip_env() {
    unsafe { std::env::set_var("MARIUS_AGENDA_SKIP_RECENT", "1") };
    with_recent_file(|json| {
        let base = std::env::temp_dir().join(format!("marius-it-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("t.marius");
        let created = create_new_project().unwrap();
        save_project_archive(&created.workspace_dir, &archive, &created.config, "t", 0).unwrap();
        assert!(!json.exists());
        let _ = fs::remove_dir_all(&created.workspace_dir);
        let _ = fs::remove_dir_all(&base);
    });
    unsafe { std::env::remove_var("MARIUS_AGENDA_SKIP_RECENT") };
}
