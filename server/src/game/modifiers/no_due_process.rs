use serde::{Deserialize, Serialize};

use crate::game::{phase::{PhaseState, PhaseStateMachine}, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct AutoGuilty;

impl From<&AutoGuilty> for ModifierID{
    fn from(_: &AutoGuilty) -> Self {
        ModifierID::AutoGuilty
    }
}

impl ModifierStateImpl for AutoGuilty{
    fn on_phase_start(self, game: &mut Game, phase: PhaseState) {
        match phase {
            PhaseState::Testimony { player_on_trial, .. }
            | PhaseState::Judgement { player_on_trial, .. } => {
                PhaseStateMachine::next_phase(game, Some(PhaseState::FinalWords { player_on_trial }))
            }
            _ => {}
        }
    }
}
