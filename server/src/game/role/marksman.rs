use serde::Serialize;

use crate::game::ability_input::AvailablePlayerListSelection;
use crate::game::attack_power::AttackPower;
use crate::game::attack_power::DefensePower;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::event::on_midnight::OnMidnightPriority;
use crate::game::game_conclusion::GameConclusion;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use crate::vec_set::VecSet;
use super::{
    ControllerID, ControllerParametersMap,
    PlayerListSelection, Role, RoleStateImpl
};

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Marksman {
    state: MarksmanState
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum MarksmanState{
    #[default]
    NotLoaded,
    Loaded,
    ShotTownie
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Marksman {
    type ClientRoleState = Marksman;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return};

        let visiting_players: Vec<_> = actor_ref
            .untagged_night_visits_cloned(midnight_variables)
            .into_iter()
            .flat_map(|p|p.target.all_night_visitors_cloned(midnight_variables))
            .collect();

        let Some(PlayerListSelection(marks)) = ControllerID::role(actor_ref, Role::Marksman, 0)
            .get_player_list_selection(game)
            .cloned() else {return};

        for mark in marks {
            if !visiting_players.contains(&mark) {continue};
            
            let killed = mark.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Marksman), AttackPower::Basic, false);

            if killed && mark.win_condition(game).is_loyalist_for(GameConclusion::Town) {
                self.state = MarksmanState::ShotTownie;
            }
        }
        
        actor_ref.set_role_state(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            // Mark
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Marksman, 0))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|
                            p.alive(game) && 
                            *p != actor_ref
                        )
                        .collect::<VecSet<_>>(),
                    can_choose_duplicates: false,
                    max_players: Some(3)
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(self.state != MarksmanState::Loaded)
                .build_map(),
            // Camp
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Marksman, 1))
                .available_selection(AvailablePlayerListSelection {
                    available_players: PlayerReference::all_players(game)
                        .filter(|p|
                            p.alive(game) && 
                            *p != actor_ref
                        )
                        .collect::<VecSet<_>>(),
                    can_choose_duplicates: false,
                    max_players: Some(3)
                })
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    self.state != MarksmanState::Loaded 
                    || ControllerID::role(actor_ref, Role::Marksman, 0)
                        .get_player_list_selection(game)
                        .is_none_or(|players| players.0.is_empty())
                )
                .build_map()
        ])
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Marksman, 1),
            false
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if 
            matches!(phase, PhaseType::Obituary) && 
            matches!(self.state, MarksmanState::NotLoaded)
        {
            actor_ref.set_role_state(game, Marksman{state: MarksmanState::Loaded})
        }
    }
}