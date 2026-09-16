use agenda_core::{create_blank_project_config, AgendaConfig};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppScreen {
    Home,
    Wizard,
}

pub(crate) struct ActiveProject {
    pub workspace_dir: PathBuf,
    pub archive_path: Option<PathBuf>,
    pub display_name: String,
    pub dirty: bool,
}

pub(crate) struct AppState {
    pub root: PathBuf,
    pub screen: AppScreen,
    pub project: Option<ActiveProject>,
    pub config: AgendaConfig,
    pub step: usize,
    pub generating: bool,
    sync: RefCell<Option<Box<dyn Fn(&mut AgendaConfig)>>>,
}

impl AppState {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            screen: AppScreen::Home,
            project: None,
            config: create_blank_project_config(),
            step: 0,
            generating: false,
            sync: RefCell::new(None),
        }
    }

    pub fn in_wizard(&self) -> bool {
        self.screen == AppScreen::Wizard && self.project.is_some()
    }

    pub fn workspace_dir(&self) -> Option<&PathBuf> {
        self.project.as_ref().map(|p| &p.workspace_dir)
    }

    pub fn set_sync(&self, sync: Option<Box<dyn Fn(&mut AgendaConfig)>>) {
        *self.sync.borrow_mut() = sync;
    }

    pub fn mark_project_dirty(&mut self) {
        if let Some(p) = self.project.as_mut() {
            p.dirty = true;
        }
    }
}

pub(crate) fn apply_step_sync(state: &Rc<RefCell<AppState>>) {
    let sync = state.borrow_mut().sync.take();
    if let Some(sync) = sync {
        let mut s = state.borrow_mut();
        sync(&mut s.config);
        s.mark_project_dirty();
    }
}

/// Mutate `AppState`, release the `RefCell` borrow, then run `after` (e.g. `refresh_step_page`).
pub(crate) fn edit_state_then(
    state: &Rc<RefCell<AppState>>,
    change: impl FnOnce(&mut AppState),
    after: impl FnOnce(),
) {
    change(&mut state.borrow_mut());
    after();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wizard_state() -> Rc<RefCell<AppState>> {
        let mut s = AppState::new(PathBuf::from("."));
        s.screen = AppScreen::Wizard;
        s.project = Some(ActiveProject {
            workspace_dir: PathBuf::from("/tmp/ws-test"),
            archive_path: None,
            display_name: "t".into(),
            dirty: false,
        });
        Rc::new(RefCell::new(s))
    }

    #[test]
    fn apply_step_sync_applies_closure_and_marks_dirty() {
        let state = wizard_state();
        state.borrow().set_sync(Some(Box::new(|cfg| {
            cfg.title = "synced".into();
        })));
        apply_step_sync(&state);
        let s = state.borrow();
        assert_eq!(s.config.title, "synced");
        assert!(s.project.as_ref().unwrap().dirty);
        assert!(state.borrow().sync.borrow().is_none());
    }

    #[test]
    fn in_wizard_requires_active_project() {
        let state = Rc::new(RefCell::new(AppState::new(PathBuf::from("."))));
        assert!(!state.borrow().in_wizard());
        state.borrow_mut().screen = AppScreen::Wizard;
        assert!(!state.borrow().in_wizard());
    }

    #[test]
    fn edit_state_then_allows_borrow_in_after_closure() {
        let state = wizard_state();
        edit_state_then(
            &state,
            |s| {
                s.config.output_dir = "/chosen/pdf".into();
                s.mark_project_dirty();
            },
            || {
                assert_eq!(state.borrow().config.output_dir, "/chosen/pdf");
                assert!(state.borrow().project.as_ref().unwrap().dirty);
            },
        );
    }

    #[test]
    fn edit_state_then_matches_refresh_step_page_pattern() {
        let state = wizard_state();
        let step_before = state.borrow().step;
        edit_state_then(
            &state,
            |s| s.config.output_dir = "out".into(),
            || {
                let _ = state.borrow().step;
            },
        );
        assert_eq!(state.borrow().step, step_before);
    }

    #[test]
    fn in_wizard_true_when_project_active() {
        let mut s = AppState::new(PathBuf::from("."));
        s.screen = AppScreen::Wizard;
        s.project = Some(ActiveProject {
            workspace_dir: PathBuf::from("/tmp/ws"),
            archive_path: None,
            display_name: "p".into(),
            dirty: false,
        });
        assert!(s.in_wizard());
    }
}
