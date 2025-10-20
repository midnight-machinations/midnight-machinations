use serde::Serialize;
use crate::game::prelude::*;
use crate::vec_map::vec_map;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Warper;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Warper {
    type ClientAbilityState = Warper;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Warper {return;}
    
        let mut transporter_visits = Visits::into_iter(midnight_variables).default_visits(actor_ref, Role::Warper);
        let Some(first_visit) = transporter_visits.next().map(|v| v.target) else {return};
        let Some(second_visit) = transporter_visits.next().map(|v| v.target) else {return};
        
        Transport::transport(
            midnight_variables, TransportPriority::Warper, 
            &vec_map![(first_visit, second_visit)], |_| true, true, 
        );
        
        actor_ref.reveal_players_role(game, first_visit);
        actor_ref.push_night_message(
            midnight_variables, ChatMessageVariant::TargetHasRole { role: first_visit.role(game) }
        );
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
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut OnMidnightFold, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}