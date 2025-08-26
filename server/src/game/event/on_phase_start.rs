use crate::game::{
    controllers::Controllers, components::{
        cult::Cult, detained::Detained, forfeit_vote::ForfeitNominationVote, mafia::Mafia, silenced::Silenced, verdicts_today::VerdictsToday
    }, modifiers::ModifierSettings, phase::PhaseState, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnPhaseStart{
    phase: PhaseState
}
impl OnPhaseStart{
    pub fn new(phase: PhaseState) -> Self{
        Self{ phase }
    }
    pub fn invoke(self, game: &mut Game){
        for player_ref in PlayerReference::all_players(game){
            player_ref.on_phase_start(game, self.phase.phase());
        }
        
        Silenced::on_phase_start(game, self.phase.phase());
        ForfeitNominationVote::on_phase_start(game, self.phase.phase());
        Detained::on_phase_start(game, self.phase.phase());
        VerdictsToday::on_phase_start(game, self.phase.phase());
        Mafia::on_phase_start(game, self.phase.phase());
        Cult::on_phase_start(game, self.phase.phase());
        Controllers::on_phase_start(game, self.phase.phase());
        ModifierSettings::on_phase_start(game, self.phase.clone());

        game.on_phase_start(self.phase.phase());
    }
}