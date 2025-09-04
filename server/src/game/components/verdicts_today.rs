use std::collections::HashSet;

use crate::game::{event::on_phase_start::OnPhaseStart, phase::PhaseType, player::PlayerReference, verdict::Verdict, Game};

#[derive(Default, Clone)]
pub struct VerdictsToday{
    guilties: HashSet<PlayerReference>,
}

impl Game{
    pub fn verdicts_today(&self)->&VerdictsToday{
        &self.verdicts_today
    }
    pub fn set_verdicts_today(&mut self, verdicts_today: VerdictsToday){
        self.verdicts_today = verdicts_today;
    }
}

impl VerdictsToday{
    pub fn new()->Self{
        Self{
            guilties: HashSet::new(),
        }
    }
    pub fn player_guiltied_today(game: &Game, player: &PlayerReference)->bool{
        game.verdicts_today().guilties.contains(player)
    }
    pub fn guilties(game: &Game)->&HashSet<PlayerReference>{
        &game.verdicts_today().guilties
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        if event.phase.phase() == PhaseType::Obituary {
            game.set_verdicts_today(VerdictsToday::new());
        }
    }
    pub fn before_phase_end(game: &mut Game, phase: PhaseType){
        if phase == PhaseType::Judgement {
            let mut verdicts_today = game.verdicts_today().clone();
            for player in PlayerReference::all_players(game) {
                if player.verdict(game) == Verdict::Guilty {
                    verdicts_today.guilties.insert(player);
                }
            }
            game.set_verdicts_today(verdicts_today);
        }
    }
}