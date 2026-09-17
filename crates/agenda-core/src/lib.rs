//! Core domain: config, school calendar, booklet imposition.

mod booklet;
mod calendar;
mod config;
mod defaults;
mod output_dir;
mod paths;
mod project;
mod store;
mod wizard;

pub use booklet::{booklet_side_class, build_imposition, pad_count_to_signature, ImposedSheet, ImposedSheetSide};
pub use calendar::{
    active_weekdays, list_agenda_days, list_school_periods, month_name_fr, parse_iso_date, period_filename,
    to_iso_date, weekday_name_fr, SchoolDay, SchoolPeriod,
};
pub use config::{
    AgendaConfig, AgendaContact, AgendaIllustrations, BookletDuplexPass, BookletOptions, HolidayPeriod,
    IllustrationSlot, IsoDate, MAX_CONTACT_PHONES, SchoolDaysMap, WeekdayIndex, contact_row_has_content,
    empty_contact, empty_slot, format_fr_phone_display, format_fr_phone_live, is_incomplete_fr_phone,
    is_valid_fr_national_phone, normalize_contact, normalized_contacts, phone_digits, slot,
    validate_contacts,
};
pub use defaults::{
    create_blank_project_config, create_default_config, weekday_label_fr, WEEKDAY_LABELS_FR,
    WEEKDAY_ORDER,
};
pub use paths::resolve_app_root;
pub use output_dir::{
    output_dir_issue_message, resolve_output_dir_for_generate, OutputDirIssue,
};
pub use project::{
    create_new_project, delete_project_archive, load_recent_projects, note_recent_open,
    open_project_archive, remove_recent_project_entry,
    prune_stale_workspace_dirs, remove_ephemeral_workspace, save_project_archive,
    write_workspace_config, NewProject, OpenProject, PROJECT_EXTENSION, ProjectError,
    RecentProjectEntry, RecentProjects,
};
pub use paths::{
    config_path, default_user_data_dir, effective_data_dir, home_dir, picker_start_dir,
    resolve_data_dir, user_home_for_file_dialog,
    resolve_output_dir,
};
pub use store::{load_config, load_config_from, save_config, save_config_to};
pub use wizard::{
    can_advance_wizard_step, format_wizard_errors, is_valid_iso_date, load_wizard_step,
    save_wizard_step, validate_wizard_step, WIZARD_STEP_COUNT,
};
