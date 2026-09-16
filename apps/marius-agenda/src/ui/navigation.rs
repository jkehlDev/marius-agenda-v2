//! Wizard navigation (GTK handlers + GUI self-test).

use super::state::{apply_step_sync, AppState};
use super::refresh_step_page;
use crate::wizard_nav::{can_goto_step, next_click_outcome, prev_click_outcome, NextClickOutcome};
use gtk::{Label, Stack};
use libadwaita as adw;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct WizardUi {
    pub app_stack: gtk::Stack,
    pub stack: Stack,
    pub step_title: Label,
    /// Full intro block (hidden from step 2 onward to save vertical space).
    pub hero_intro: gtk::Box,
    pub step_scrolled: gtk::ScrolledWindow,
    pub window: adw::ApplicationWindow,
    pub home_btn: gtk::Button,
    pub save_menu_btn: gtk::MenuButton,
    pub prev_btn: gtk::Button,
    pub next_btn: gtk::Button,
    pub step_errors: Label,
    pub step_pills: Vec<gtk::Button>,
    pub gen_banner: adw::Banner,
    pub home_recent_list: gtk::Box,
    /// Last step index whose body widgets were built (avoids full rebuild on navigation).
    pub last_filled_step: Cell<Option<usize>>,
}

impl WizardUi {
    pub fn clone_handles(&self) -> Self {
        Self {
            app_stack: self.app_stack.clone(),
            stack: self.stack.clone(),
            step_title: self.step_title.clone(),
            hero_intro: self.hero_intro.clone(),
            step_scrolled: self.step_scrolled.clone(),
            window: self.window.clone(),
            home_btn: self.home_btn.clone(),
            save_menu_btn: self.save_menu_btn.clone(),
            prev_btn: self.prev_btn.clone(),
            next_btn: self.next_btn.clone(),
            step_errors: self.step_errors.clone(),
            step_pills: self.step_pills.clone(),
            gen_banner: self.gen_banner.clone(),
            home_recent_list: self.home_recent_list.clone(),
            last_filled_step: Cell::new(self.last_filled_step.get()),
        }
    }
}

fn set_step(state: &Rc<RefCell<AppState>>, step: usize) {
    state.borrow_mut().step = step;
}

fn try_advance(state: &Rc<RefCell<AppState>>) -> bool {
    let s = state.borrow();
    agenda_core::can_advance_wizard_step(s.step, &s.config)
}

pub fn perform_wizard_goto(ui: &WizardUi, state: &Rc<RefCell<AppState>>, target: usize) {
    let current = state.borrow().step;
    if !can_goto_step(current, target) || target == current {
        return;
    }
    apply_step_sync(state);
    set_step(state, target);
    refresh_step_page(ui, state, false);
}

pub fn perform_wizard_prev(ui: &WizardUi, state: &Rc<RefCell<AppState>>) {
    apply_step_sync(state);
    let next = prev_click_outcome(state.borrow().step);
    set_step(state, next);
    refresh_step_page(ui, state, false);
}

pub fn perform_wizard_next(ui: &WizardUi, state: &Rc<RefCell<AppState>>) -> NextClickOutcome {
    apply_step_sync(state);
    super::update_wizard_errors(ui, state);
    if !try_advance(state) {
        return NextClickOutcome::Blocked;
    }

    let outcome = {
        let mut s = state.borrow_mut();
        let step = s.step;
        match next_click_outcome(step, true) {
            NextClickOutcome::Generate => NextClickOutcome::Generate,
            NextClickOutcome::Advanced(to) => {
                s.step = to;
                NextClickOutcome::Advanced(to)
            }
            NextClickOutcome::Blocked => NextClickOutcome::Blocked,
        }
    };

    if matches!(outcome, NextClickOutcome::Advanced(_)) {
        refresh_step_page(ui, state, false);
    }
    outcome
}
