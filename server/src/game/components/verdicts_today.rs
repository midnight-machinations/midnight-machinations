use crate::{game::{event::{before_phase_end::BeforePhaseEnd, on_phase_start::OnPhaseStart}, phase::{PhaseState, PhaseType}, player::PlayerReference, verdict::Verdict, Game}, vec_map::VecMap, vec_set::VecSet};

#[derive(Default, Clone)]
pub struct VerdictsToday{
    guilties: VecMap<PlayerReference, VecSet<PlayerReference>>,
}

impl VerdictsToday{
    pub fn new()->Self{
        Self{
            guilties: VecMap::new(),
        }
    }
    pub fn guilties_during_any_trial(game: &Game)->VecSet<PlayerReference>{
        game.verdicts_today.guilties
            .iter()
            .flat_map(|(_,players)|
                players
                .iter()
                .copied()
            )
            .collect()
    }
    pub fn guilties_during_trial(game: &Game, player_on_trial: PlayerReference)->VecSet<PlayerReference>{
        game.verdicts_today.guilties.get(&player_on_trial).cloned().unwrap_or_default()
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        if event.phase.phase() == PhaseType::Obituary {
            game.verdicts_today = VerdictsToday::new();
        }
    }
    pub fn player_was_on_trial(game: &Game, player_on_trial: PlayerReference)->bool{
        game.verdicts_today.guilties.contains(&player_on_trial)
    }
    pub fn before_phase_end(game: &mut Game, event: &BeforePhaseEnd, _fold: &mut (), _priority: ()){
        if event.phase != PhaseType::Judgement {return;}
        let PhaseState::Judgement{
            player_on_trial,
            ..
        } = game.current_phase() else {return};

        game.verdicts_today.guilties.insert(
            *player_on_trial,
            game
                .verdicts_today
                .guilties
                .get(player_on_trial)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .chain(
                    PlayerReference::all_players(game).filter(|p|p.verdict(game)==Verdict::Guilty)
                ).collect()
        );
    }
}