use crate::game::{
    event::EventData, phase::PhaseState,
};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnPhaseStart{
    pub phase: PhaseState
}
impl OnPhaseStart{
    pub fn new(phase: PhaseState) -> (Self, ()){
        (Self{ phase }, ())
    }
}
impl EventData for OnPhaseStart{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnPhaseStart{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::components::blocked::BlockedComponent::on_phase_start,
        crate::game::abilities_component::Abilities::on_phase_start,
        crate::game::components::forfeit_vote::ForfeitNominationVote::on_phase_start,
        crate::game::components::detained::Detained::on_phase_start,
        crate::game::components::verdicts_today::VerdictsToday::on_phase_start,
        crate::game::controllers::Controllers::on_phase_start,
        crate::game::modifiers::ModifierSettings::on_phase_start,
        crate::game::components::silenced::Silenced::on_phase_start,
        crate::game::components::call_witness::CallWitness::on_phase_start,
        crate::game::Game::on_phase_start,
        crate::game::components::fast_forward::FastForwardComponent::on_phase_start,
        crate::game::components::ascend::Ascend::on_phase_start,
    ]}
}