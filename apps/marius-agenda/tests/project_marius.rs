use agenda_core::{
    create_new_project, open_project_archive, remove_ephemeral_workspace, save_project_archive,
    PROJECT_EXTENSION,
};
use std::fs;

#[test]
fn save_and_open_marius_archive() {
    unsafe { std::env::set_var("MARIUS_AGENDA_SKIP_RECENT", "1") };

    let base = std::env::temp_dir().join(format!("marius-it-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    let archive = base.join(format!("test.{}", PROJECT_EXTENSION));

    let created = create_new_project().unwrap();
    let mut cfg = created.config.clone();
    cfg.title = "Integration test".into();
    save_project_archive(&created.workspace_dir, &archive, &cfg, "Integration test", 2).unwrap();

    let opened = open_project_archive(&archive).unwrap();
    assert_eq!(opened.wizard_step, 2);
    assert_eq!(opened.config.title, "Integration test");
    assert!(opened.config.illustrations.cover.source_path.is_none());

    remove_ephemeral_workspace(&created.workspace_dir);
    remove_ephemeral_workspace(&opened.workspace_dir);
    let _ = fs::remove_dir_all(&base);
}
