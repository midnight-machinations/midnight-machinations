use serde::{Deserialize, Serialize};

use crate::game::{phase::{PhaseState, PhaseStateMachine}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoTrialPhases;

impl From<&NoTrialPhases> for ModifierID{
    fn from(_: &NoTrialPhases) -> Self {
        ModifierID::NoTrialPhases
    }
}

impl ModifierStateImpl for NoTrialPhases{
    fn on_phase_start(self, game: &mut Game, phase: PhaseState) {
        match phase {
            PhaseState::Nomination { .. }
            | PhaseState::Testimony { .. }
            | PhaseState::Judgement { .. } => {
                PhaseStateMachine::next_phase(game, Some(PhaseState::Dusk))
            }
            _ => {}
        }
    }
}
