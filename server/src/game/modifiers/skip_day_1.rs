use serde::{Deserialize, Serialize};

use crate::game::{
    components::fast_forward::FastForwardComponent, event::on_phase_start::OnPhaseStart,
    phase::PhaseType::*, Game
};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SkipDay1;

impl From<&SkipDay1> for ModifierID{
    fn from(_: &SkipDay1) -> Self {
        ModifierID::SkipDay1
    }
}

impl ModifierStateImpl for SkipDay1{
    fn on_phase_start(self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
        match (event.phase.phase(), game.day_number()) {
            (Dusk, 1) |
            (Night, 1) |
            (Obituary, 2) |
            (Discussion, 2) |
            (Nomination, 2) |
            (Testimony, 2) |
            (Judgement, 2) |
            (FinalWords, 2)
                => FastForwardComponent::skip(game),
            _ => ()
        }
    }
}
