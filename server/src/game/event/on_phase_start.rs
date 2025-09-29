use crate::game::{
    abilities_component::Abilities,
    components::{
        call_witness::CallWitness, detained::Detained,
        fast_forward::FastForwardComponent, forfeit_vote::ForfeitNominationVote,
        silenced::Silenced, verdicts_today::VerdictsToday
    },
    controllers::Controllers, event::Event, modifiers::ModifierSettings, phase::PhaseState,
    Game
};


#[must_use = "Event must be invoked"]
pub struct OnPhaseStart{
    pub phase: PhaseState
}
impl OnPhaseStart{
    pub fn new(phase: PhaseState) -> Self{
        Self{ phase }
    }
}
impl Event for OnPhaseStart{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_phase_start,
        ForfeitNominationVote::on_phase_start,
        Detained::on_phase_start,
        VerdictsToday::on_phase_start,
        Controllers::on_phase_start,
        ModifierSettings::on_phase_start,
        Silenced::on_phase_start,   //silenced needs to go before call witness, I could do priority but erm
        CallWitness::on_phase_start,    //must go after silenced
        Game::on_phase_start,
        FastForwardComponent::on_phase_start,
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}