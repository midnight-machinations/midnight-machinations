use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::components::night_visits::{NightVisitsIterator as _, Visits};
use crate::game::controllers::AvailableTwoPlayerOptionSelection;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::components::transport::{Transport, TransportPriority};
use crate::vec_set;

use crate::vec_map::vec_map;

use super::{common_role, ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Transporter;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Transporter {
    type ClientAbilityState = Transporter;
    fn on_midnight(self, _game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Transporter {return;}
    
        let mut targets = Visits::into_iter(midnight_variables).default_targets(actor_ref, Role::Transporter);
        let Some(a) = targets.next() else {return};
        let Some(b) = targets.next() else {return};
        
        Transport::transport(
            midnight_variables, TransportPriority::Transporter, 
            &vec_map![(a, b), (b, a)], |_| true, true
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        let available_players: vec_set::VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p| p.alive(game))
            .collect();

        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Transporter, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: available_players.clone(),
                available_second_players: available_players,
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
            ControllerID::role(actor_ref, Role::Transporter, 0),
            false
        )
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}