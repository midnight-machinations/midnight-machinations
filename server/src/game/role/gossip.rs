use serde::Serialize;

use crate::game::components::confused::Confused;
use crate::game::components::night_visits::{NightVisitsIterator as _, Visits};
use crate::game::event::on_midnight::{OnMidnightFold, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use crate::game::abilities_component::ability_id::AbilityID;
use super::detective::Detective;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Gossip;

impl RoleStateTrait for Gossip {
    type ClientAbilityState = Gossip;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        if let Some(visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Gossip) {
            
            let enemies = if Confused::is_confused(game, actor_ref){
                false
            }else{
                Gossip::enemies_night(game, midnight_variables, visit.target)
            };
            
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::GossipResult{ enemies });
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Gossip, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Gossip, 0),
            false
        )
    }
}

impl Gossip {
    pub fn enemies_night(game: &Game, midnight_variables: &mut OnMidnightFold, player_ref: PlayerReference) -> bool {
        Visits::into_iter(midnight_variables)
            .with_visitor(player_ref)
            .map_target()
            .any(|targets_target|
                Detective::player_is_suspicious(game, midnight_variables, targets_target)
            )
    }
}