//! Wizard + config regression tests (no GTK).

use agenda_core::{
    can_advance_wizard_step, create_default_config, format_wizard_errors, load_wizard_step,
    save_wizard_step, validate_wizard_step, WIZARD_STEP_COUNT,
};

#[test]
fn default_config_has_no_validation_errors_per_step() {
    let config = create_default_config();
    for step in 0..WIZARD_STEP_COUNT {
        assert!(
            validate_wizard_step(step, &config).is_empty(),
            "step {}: {:?}",
            step,
            validate_wizard_step(step, &config)
        );
    }
}

#[test]
fn wizard_step_file_survives_reload() {
    let base = std::env::temp_dir().join(format!("marius-wiz-file-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("assets")).unwrap();
    std::fs::create_dir_all(base.join("fonts")).unwrap();
    std::fs::create_dir_all(base.join("data")).unwrap();

    save_wizard_step(&base, 2).unwrap();
    assert_eq!(load_wizard_step(&base), 2);
    save_wizard_step(&base, 99).unwrap();
    assert_eq!(load_wizard_step(&base), 2, "out-of-range step must not overwrite file");

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn empty_title_blocks_step_zero_and_formats_errors() {
    let mut config = create_default_config();
    config.title = "   ".into();
    let errs = validate_wizard_step(0, &config);
    assert!(!can_advance_wizard_step(0, &config));
    let formatted = format_wizard_errors(&errs);
    assert!(formatted.contains('•'));
}

#[test]
fn illustrations_step_is_optional() {
    let config = create_default_config();
    assert!(can_advance_wizard_step(3, &config));
}
