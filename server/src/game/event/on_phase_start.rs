use crate::game::{event::EventData, prelude::*};

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
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        BlockedComponent::on_phase_start,
        Abilities::on_event,
        ForfeitNominationVote::on_phase_start,
        Detained::on_phase_start,
        VerdictsToday::on_phase_start,
        ModifierSettings::on_phase_start,
        Silenced::on_phase_start,   //silenced needs to go before call witness, I could do priority but erm
        CallWitness::on_phase_start,    //must go after silenced
        Controllers::on_phase_start,
        Game::on_phase_start,
        FastForwardComponent::on_phase_start,
        Ascend::on_phase_start,
    ]}
}