use serde::Serialize;

use crate::game::controllers::AvailableTwoPlayerOptionSelection;
use crate::game::components::graves::grave::Grave;
use crate::game::components::transport::{Transport, TransportPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;

use crate::game::components::win_condition::WinCondition;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;

use crate::vec_map::vec_map;

use super::{common_role, ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Warper;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Warper {
    type ClientRoleState = Warper;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Warper {return;}
    
        let transporter_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(first_visit) = transporter_visits.get(0).map(|v| v.target) else {return};
        let Some(second_visit) = transporter_visits.get(1).map(|v| v.target) else {return};
        
        Transport::transport(
            midnight_variables, TransportPriority::Warper, 
            &vec_map![(first_visit, second_visit)], |_| true, true, 
        );
        
        actor_ref.push_night_message(
            midnight_variables, ChatMessageVariant::TargetHasRole { role: first_visit.role(game) }
        );
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        if
            actor_ref.alive(game) &&
            PlayerReference::all_players(game)
                .filter(|p|p.alive(game))
                .filter(|p|p.keeps_game_running(game))
                .all(|p|
                    WinCondition::are_friends(p.win_condition(game), actor_ref.win_condition(game))
                )

        {
            actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Warper, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .filter(|p|*p != actor_ref)
                    .collect(),
                available_second_players:PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .collect(),
                can_choose_duplicates: false,
                can_choose_none: true
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Warper, 0),
            false
        )
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}