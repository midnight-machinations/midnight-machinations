use serde::Serialize;

use crate::game::controllers::AvailableTwoPlayerOptionSelection;
use crate::game::components::transport::{Transport, TransportPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;

use crate::vec_map::vec_map;

use super::{common_role, ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Porter;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Porter {
    type ClientAbilityState = Porter;
    fn on_midnight(self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Warper {return;}
    
        let transporter_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(first_visit) = transporter_visits.get(0).map(|v| v.target) else {return};
        let Some(second_visit) = transporter_visits.get(1).map(|v| v.target) else {return};
        
        Transport::transport(
            midnight_variables, TransportPriority::Warper, 
            &vec_map![(first_visit, second_visit)], |_| true, true
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Porter, 0))
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
            ControllerID::role(actor_ref, Role::Porter, 0),
            false
        )
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}