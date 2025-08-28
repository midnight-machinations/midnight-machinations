use crate::game::{components::fast_forward::FastForwardComponent, phase::{PhaseState, PhaseType::*}, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct SkipDay1;

impl From<&SkipDay1> for ModifierType{
    fn from(_: &SkipDay1) -> Self {
        ModifierType::SkipDay1
    }
}

impl ModifierTrait for SkipDay1{
    fn on_phase_start(self, game: &mut Game, phase: PhaseState) {
        match (phase.phase(), game.day_number()) {
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
