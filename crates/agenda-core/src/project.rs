//! `.marius` project archives (ZIP) — sole on-disk project format for the GUI.

use crate::config::AgendaConfig;
use crate::create_blank_project_config;
use crate::wizard::WIZARD_STEP_COUNT;
use crate::paths::{config_path, default_user_data_dir};
use crate::store::{load_config_from, save_config_to};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use time::OffsetDateTime;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub const PROJECT_FORMAT_VERSION: u32 = 1;
pub const PROJECT_EXTENSION: &str = "marius";

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid project: {0}")]
    Invalid(String),
    #[error("project not saved yet")]
    NotSaved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub app: String,
    pub saved_at: String,
    pub title: String,
    /// Last wizard step index when the project was saved (0-based).
    #[serde(default)]
    pub wizard_step: u32,
}

#[derive(Debug, Clone)]
pub struct OpenProject {
    pub workspace_dir: PathBuf,
    pub archive_path: PathBuf,
    pub config: AgendaConfig,
    pub display_name: String,
    pub wizard_step: usize,
}

#[derive(Debug, Clone)]
pub struct NewProject {
    pub workspace_dir: PathBuf,
    pub config: AgendaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProjectEntry {
    pub path: PathBuf,
    pub title: String,
    pub opened_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentProjects {
    pub entries: Vec<RecentProjectEntry>,
}

pub fn workspaces_cache_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::paths::home_dir().join(".cache"));
    base.join("marius-agenda").join("workspaces")
}

pub fn recent_projects_path() -> PathBuf {
    if let Ok(p) = std::env::var("MARIUS_AGENDA_RECENT_PROJECTS") {
        return PathBuf::from(p);
    }
    default_user_data_dir()
        .parent()
        .map(|p| p.join("recent-projects.json"))
        .unwrap_or_else(|| PathBuf::from("recent-projects.json"))
}

pub fn create_new_project() -> Result<NewProject, ProjectError> {
    let workspace_dir = fresh_workspace_dir()?;
    let config = create_blank_project_config();
    save_config_to(Path::new("."), &workspace_dir, &config)?;
    Ok(NewProject { workspace_dir, config })
}

pub fn open_project_archive(archive_path: &Path) -> Result<OpenProject, ProjectError> {
    let archive_path = archive_path.canonicalize()?;
    if archive_path.extension().and_then(|e| e.to_str()) != Some(PROJECT_EXTENSION) {
        return Err(ProjectError::Invalid(format!(
            "expected .{} file",
            PROJECT_EXTENSION
        )));
    }
    let workspace_dir = fresh_workspace_dir()?;
    unpack_archive(&archive_path, &workspace_dir)?;
    let config = load_config_from(&workspace_dir)?;
    let manifest = read_manifest(&workspace_dir)?;
    let wizard_step = normalize_wizard_step(manifest.wizard_step as usize);
    let display_name = archive_path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&config.title)
        .to_string();
    Ok(OpenProject {
        workspace_dir,
        archive_path,
        config,
        display_name,
        wizard_step,
    })
}

pub fn write_workspace_config(
    workspace_dir: &Path,
    config: &AgendaConfig,
) -> Result<(), ProjectError> {
    save_config_to(Path::new("."), workspace_dir, config).map_err(ProjectError::from)
}

pub fn save_project_archive(
    workspace_dir: &Path,
    archive_path: &Path,
    config: &AgendaConfig,
    title: &str,
    wizard_step: usize,
) -> Result<(), ProjectError> {
    write_workspace_config(workspace_dir, config)?;
    let manifest = ProjectManifest {
        format_version: PROJECT_FORMAT_VERSION,
        app: "marius-agenda".into(),
        saved_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| String::new()),
        title: title.to_string(),
        wizard_step: normalize_wizard_step(wizard_step) as u32,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(workspace_dir.join("manifest.json"), manifest_json)?;
    pack_workspace(workspace_dir, archive_path)?;
    if should_update_recent_list() {
        touch_recent(archive_path, title);
    }
    Ok(())
}

pub fn load_recent_projects() -> RecentProjects {
    let path = recent_projects_path();
    let mut list = match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(_) => return RecentProjects::default(),
    };
    if prune_stale_recent_entries(&mut list) {
        persist_recent_projects(&list);
    }
    list
}

fn prune_stale_recent_entries(list: &mut RecentProjects) -> bool {
    let before = list.entries.len();
    list.entries.retain(|e| recent_archive_still_valid(&e.path));
    before != list.entries.len()
}

/// Test / scratch archives under `/tmp/marius-it-*` or `/tmp/marius-proj-*` must not pollute recents.
fn recent_archive_still_valid(path: &Path) -> bool {
    if is_test_scratch_project_archive(path) {
        cleanup_test_scratch_archive(path);
        return false;
    }
    path.is_file()
}

fn is_test_scratch_project_archive(path: &Path) -> bool {
    let parent = path.parent();
    let parent = parent.and_then(|p| p.file_name());
    let name = parent.and_then(|n| n.to_str()).unwrap_or_default();
    if !name.starts_with("marius-it-") && !name.starts_with("marius-proj-") {
        return false;
    }
    path.parent()
        .is_some_and(|p| p.starts_with(std::env::temp_dir()))
}

fn cleanup_test_scratch_archive(archive: &Path) {
    if let Some(dir) = archive.parent() {
        if dir.starts_with(std::env::temp_dir()) {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

fn persist_recent_projects(list: &RecentProjects) {
    if let Some(parent) = recent_projects_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(list) {
        let _ = fs::write(recent_projects_path(), json);
    }
}

fn touch_recent(archive_path: &Path, title: &str) {
    let mut list = load_recent_projects();
    let path = archive_path.to_path_buf();
    list.entries.retain(|e| e.path != path);
    list.entries.insert(
        0,
        RecentProjectEntry {
            path,
            title: title.to_string(),
            opened_at: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| String::new()),
        },
    );
    list.entries.truncate(8);
    persist_recent_projects(&list);
}

pub fn note_recent_open(archive_path: &Path, title: &str) {
    if should_update_recent_list() {
        touch_recent(archive_path, title);
    }
}

/// Drops one path from `recent-projects.json` (no-op if absent).
pub fn remove_recent_project_entry(archive_path: &Path) {
    let mut list = load_recent_projects();
    let before = list.entries.len();
    list.entries.retain(|e| e.path != archive_path);
    if list.entries.len() != before {
        persist_recent_projects(&list);
    }
}

/// Deletes the `.marius` file on disk and removes it from recents.
pub fn delete_project_archive(archive_path: &Path) -> Result<(), ProjectError> {
    if !archive_path.is_file() {
        remove_recent_project_entry(archive_path);
        return Err(ProjectError::Invalid(format!(
            "file not found: {}",
            archive_path.display()
        )));
    }
    fs::remove_file(archive_path)?;
    remove_recent_project_entry(archive_path);
    Ok(())
}

/// Removes a cache workspace directory (only under `workspaces_cache_dir()`).
pub fn remove_ephemeral_workspace(workspace_dir: &Path) {
    let base = workspaces_cache_dir();
    if workspace_dir.starts_with(&base) && workspace_dir.is_dir() {
        let _ = fs::remove_dir_all(workspace_dir);
    }
}

/// Deletes `ws-{pid}-*` dirs left by previous app runs (other PIDs).
pub fn prune_stale_workspace_dirs() {
    let base = workspaces_cache_dir();
    let entries = match fs::read_dir(&base) {
        Ok(e) => e,
        Err(_) => return,
    };
    let my_pid = std::process::id().to_string();
    for ent in entries.flatten() {
        let path = ent.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !name.starts_with("ws-") {
            continue;
        }
        let pid = name
            .strip_prefix("ws-")
            .and_then(|rest| rest.split('-').next())
            .unwrap_or_default();
        if pid != my_pid {
            let _ = fs::remove_dir_all(&path);
        }
    }
}

fn should_update_recent_list() -> bool {
    !cfg!(test) && std::env::var_os("MARIUS_AGENDA_SKIP_RECENT").is_none()
}

fn read_manifest(workspace_dir: &Path) -> Result<ProjectManifest, ProjectError> {
    let manifest_path = workspace_dir.join("manifest.json");
    serde_json::from_str(&fs::read_to_string(manifest_path)?).map_err(ProjectError::from)
}

pub fn normalize_wizard_step(step: usize) -> usize {
    step.min(WIZARD_STEP_COUNT.saturating_sub(1))
}

fn fresh_workspace_dir() -> Result<PathBuf, ProjectError> {
    let base = workspaces_cache_dir();
    fs::create_dir_all(&base)?;
    let id = format!(
        "ws-{}-{}",
        std::process::id(),
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    );
    let dir = base.join(id);
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(dir.join("images/print"))?;
    Ok(dir)
}

fn pack_workspace(workspace_dir: &Path, dest: &Path) -> Result<(), ProjectError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let opts =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in walk_files(workspace_dir)? {
        let rel = entry
            .strip_prefix(workspace_dir)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let name = rel.to_string_lossy().replace('\\', "/");
        zip.start_file(name, opts)?;
        let mut f = File::open(&entry)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        zip.write_all(&buf)?;
    }
    zip.finish()?;
    Ok(())
}

fn unpack_archive(archive_path: &Path, dest: &Path) -> Result<(), ProjectError> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();
        if name.contains("..") {
            return Err(ProjectError::Invalid("zip path traversal".into()));
        }
        let out = dest.join(name);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out)?;
            io::copy(&mut entry, &mut out_file)?;
        }
    }
    let manifest_path = dest.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(ProjectError::Invalid("missing manifest.json".into()));
    }
    let manifest = read_manifest(dest)?;
    if manifest.format_version != PROJECT_FORMAT_VERSION {
        return Err(ProjectError::Invalid(format!(
            "unsupported format version {}",
            manifest.format_version
        )));
    }
    if !config_path(dest).is_file() {
        return Err(ProjectError::Invalid("missing config.json".into()));
    }
    Ok(())
}

fn walk_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut out = Vec::new();
    walk_files_rec(dir, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_files_rec(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), io::Error> {
    for ent in fs::read_dir(dir)? {
        let ent = ent?;
        let path = ent.path();
        if path.is_dir() {
            walk_files_rec(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn set_env(key: &str, value: &str) {
        // Single-threaded `cargo test`; env overrides for isolated recent-list paths.
        unsafe { env::set_var(key, value) };
    }

    fn clear_env(key: &str) {
        unsafe { env::remove_var(key) };
    }

    fn unique_temp_json(label: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "marius-{}-{}-{}",
            label,
            std::process::id(),
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        ))
    }

    fn with_recent_json<F: FnOnce(&PathBuf)>(label: &str, f: F) {
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _guard = LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let json = unique_temp_json(label);
        let _ = fs::remove_file(&json);
        set_env("MARIUS_AGENDA_RECENT_PROJECTS", json.to_string_lossy().as_ref());
        f(&json);
        let _ = fs::remove_file(&json);
        clear_env("MARIUS_AGENDA_RECENT_PROJECTS");
    }

    #[test]
    fn new_project_has_blank_illustrations() {
        let created = create_new_project().unwrap();
        let cfg = load_config_from(&created.workspace_dir).unwrap();
        assert!(cfg.illustrations.cover.source_path.is_none());
        assert!(cfg.illustrations.activities.source_path.is_none());
    }

    #[test]
    fn marius_roundtrip() {
        let base = env::temp_dir().join(format!("marius-proj-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test-agenda.marius");

        let created = create_new_project().unwrap();
        let mut cfg = created.config.clone();
        cfg.title = "Test titre".into();
        let out_dir = base.join("pdf-out");
        fs::create_dir_all(&out_dir).unwrap();
        cfg.output_dir = out_dir.to_string_lossy().into_owned();
        save_project_archive(&created.workspace_dir, &archive, &cfg, "Test titre", 3).unwrap();

        let opened = open_project_archive(&archive).unwrap();
        assert_eq!(opened.wizard_step, 3);
        assert_eq!(opened.config.title, "Test titre");
        assert_eq!(opened.config.output_dir, cfg.output_dir);
        assert!(opened.archive_path.is_file());

        let _ = fs::remove_dir_all(&created.workspace_dir);
        let _ = fs::remove_dir_all(&opened.workspace_dir);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn prune_recent_drops_test_scratch_under_tmp() {
        let base = env::temp_dir().join(format!("marius-it-prune-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let archive = base.join("test.marius");
        fs::write(&archive, b"x").unwrap();
        let mut list = RecentProjects {
            entries: vec![RecentProjectEntry {
                path: archive.clone(),
                title: "scratch".into(),
                opened_at: String::new(),
            }],
        };
        assert!(prune_stale_recent_entries(&mut list));
        assert!(list.entries.is_empty());
        assert!(!base.exists());
    }

    #[test]
    fn prune_recent_drops_missing_archive_paths() {
        let mut list = RecentProjects {
            entries: vec![
                RecentProjectEntry {
                    path: PathBuf::from("/nonexistent/marius-agenda-missing.marius"),
                    title: "gone".into(),
                    opened_at: String::new(),
                },
            ],
        };
        assert!(prune_stale_recent_entries(&mut list));
        assert!(list.entries.is_empty());
    }

    #[test]
    fn delete_project_archive_removes_file_and_recent_entry() {
        with_recent_json("recent-delete", |json| {
            let base = env::temp_dir().join(format!("marius-del-archive-{}", std::process::id()));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).unwrap();
            let archive = base.join("gone.marius");

            let created = create_new_project().unwrap();
            save_project_archive(&created.workspace_dir, &archive, &created.config, "gone", 0)
                .unwrap();
            let path_json = serde_json::to_string(&archive).unwrap();
            fs::write(
                json,
                format!(
                    r#"{{"entries":[{{"path":{},"title":"gone","opened_at":""}}]}}"#,
                    path_json
                ),
            )
            .unwrap();
            assert_eq!(load_recent_projects().entries.len(), 1);

            delete_project_archive(&archive).unwrap();
            assert!(!archive.exists());
            assert!(load_recent_projects().entries.is_empty());

            let _ = fs::remove_dir_all(&created.workspace_dir);
            let _ = fs::remove_dir_all(&base);
        });
    }

    #[test]
    fn load_recent_persists_prune_to_json_file() {
        with_recent_json("recent-prune", |json| {
            fs::write(
                json,
                r#"{"entries":[{"path":"/nonexistent/ghost.marius","title":"x","opened_at":""}]}"#,
            )
            .unwrap();
            let list = load_recent_projects();
            assert!(list.entries.is_empty());
            let raw = fs::read_to_string(json).unwrap();
            assert!(!raw.contains("ghost.marius"));
        });
    }

    #[test]
    fn save_skips_recent_when_env_skip_set() {
        with_recent_json("recent-skip", |json| {
            set_env("MARIUS_AGENDA_SKIP_RECENT", "1");

            let base = env::temp_dir().join(format!("marius-proj-skip-{}", std::process::id()));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).unwrap();
            let archive = base.join("skip.marius");

            let created = create_new_project().unwrap();
            save_project_archive(&created.workspace_dir, &archive, &created.config, "skip", 0)
                .unwrap();
            assert!(!json.exists());

            let _ = fs::remove_dir_all(&created.workspace_dir);
            let _ = fs::remove_dir_all(&base);
            clear_env("MARIUS_AGENDA_SKIP_RECENT");
        });
    }

    #[test]
    fn remove_ephemeral_workspace_only_under_cache() {
        let outside = env::temp_dir().join(format!("marius-ws-outside-{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();
        remove_ephemeral_workspace(&outside);
        assert!(outside.is_dir());
        let _ = fs::remove_dir_all(&outside);

        let inside = workspaces_cache_dir().join(format!("ws-999999999-test-{}", std::process::id()));
        fs::create_dir_all(&inside).unwrap();
        remove_ephemeral_workspace(&inside);
        assert!(!inside.exists());
    }

    #[test]
    fn prune_stale_workspace_dirs_removes_other_pid_dirs() {
        let base = workspaces_cache_dir();
        fs::create_dir_all(&base).unwrap();
        let stale = base.join(format!("ws-999999999-stale-{}", std::process::id()));
        fs::create_dir_all(&stale).unwrap();
        prune_stale_workspace_dirs();
        assert!(!stale.exists());
    }
}
