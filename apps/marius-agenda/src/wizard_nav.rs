//! Wizard step navigation (pure logic, testable without GTK).

use agenda_core::WIZARD_STEP_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NextClickOutcome {
    Blocked,
    Advanced(usize),
    Generate,
}

/// Result of a « Suivant » click when validation already passed.
pub fn next_click_outcome(current_step: usize, validation_ok: bool) -> NextClickOutcome {
    if !validation_ok {
        return NextClickOutcome::Blocked;
    }
    if current_step >= WIZARD_STEP_COUNT - 1 {
        NextClickOutcome::Generate
    } else {
        NextClickOutcome::Advanced(current_step + 1)
    }
}

pub fn prev_click_outcome(current_step: usize) -> usize {
    current_step.saturating_sub(1)
}

/// Pills only allow jumping to steps already reached (index <= current).
pub fn can_goto_step(current_step: usize, target: usize) -> bool {
    target < WIZARD_STEP_COUNT && target <= current_step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_advances_through_first_four_steps() {
        for step in 0..4 {
            assert_eq!(
                next_click_outcome(step, true),
                NextClickOutcome::Advanced(step + 1)
            );
        }
    }

    #[test]
    fn next_on_last_step_triggers_generate() {
        assert_eq!(
            next_click_outcome(4, true),
            NextClickOutcome::Generate
        );
    }

    #[test]
    fn next_blocked_when_invalid() {
        assert_eq!(next_click_outcome(0, false), NextClickOutcome::Blocked);
    }

    #[test]
    fn prev_never_panics_and_stops_at_zero() {
        assert_eq!(prev_click_outcome(0), 0);
        assert_eq!(prev_click_outcome(3), 2);
    }

    #[test]
    fn goto_only_to_revealed_steps() {
        assert!(can_goto_step(3, 0));
        assert!(can_goto_step(3, 3));
        assert!(!can_goto_step(3, 4));
        assert!(!can_goto_step(0, 1));
    }

    #[test]
    fn default_config_advances_through_wizard() {
        use agenda_core::{can_advance_wizard_step, create_default_config};
        let config = create_default_config();
        for step in 0..4 {
            assert!(
                can_advance_wizard_step(step, &config),
                "step {}: should advance",
                step
            );
            assert_eq!(
                next_click_outcome(step, true),
                NextClickOutcome::Advanced(step + 1)
            );
        }
        assert_eq!(
            next_click_outcome(4, true),
            NextClickOutcome::Generate
        );
    }
}
