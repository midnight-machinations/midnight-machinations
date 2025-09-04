use serde::{Deserialize, Serialize};

use crate::game::{event::on_phase_start::OnPhaseStart, phase::{PhaseState, PhaseStateMachine}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoTrialPhases;

impl From<&NoTrialPhases> for ModifierID{
    fn from(_: &NoTrialPhases) -> Self {
        ModifierID::NoTrialPhases
    }
}

impl ModifierStateImpl for NoTrialPhases{
    fn on_phase_start(self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
        match event.phase {
            PhaseState::Nomination { .. }
            | PhaseState::Adjournment { .. }
            | PhaseState::Testimony { .. }
            | PhaseState::Judgement { .. } => {
                PhaseStateMachine::next_phase(game, Some(PhaseState::Dusk))
            }
            _ => {}
        }
    }
}
