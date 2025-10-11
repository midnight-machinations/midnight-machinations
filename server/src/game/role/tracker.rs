use rand::prelude::SliceRandom;
use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Serialize, Debug, Default)]
pub struct Tracker;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Tracker {
    type ClientAbilityState = Tracker;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let Some(target) = Visits::default_target(game, midnight_variables, actor_ref) else {return};
            
        let mut players: Vec<PlayerReference> = target.tracker_seen_players(midnight_variables).collect();
        players.shuffle(&mut rand::rng());
        
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::TrackerResult { players });
        
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Tracker, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Tracker, 0),
            false
        )
    }
}