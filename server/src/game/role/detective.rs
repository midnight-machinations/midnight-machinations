use serde::Serialize;

use crate::game::controllers::ControllerID;
use crate::game::components::aura::Aura;
use crate::game::components::confused::Confused;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;
use super::{ControllerParametersMap, Role, RoleStateImpl};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Detective;

impl RoleStateImpl for Detective {
    type ClientRoleState = Detective;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}
        
        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){
            let suspicious = if Confused::is_confused(game, actor_ref) {
                false
            }else{
                Detective::player_is_suspicious(game, midnight_variables, visit.target)
            };

            let message = ChatMessageVariant::SheriffResult {
                suspicious
            };
            
            actor_ref.push_night_message(midnight_variables, message);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Detective, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Detective, 0),
            false
        )
    }
}

impl Detective {
    pub fn player_is_suspicious(game: &Game, midnight_variables: &MidnightVariables, player_ref: PlayerReference) -> bool {
        if Aura::suspicious(game, midnight_variables, player_ref){
            true
        }else if Aura::innocent(game, player_ref){
            false
        }else{
            !player_ref.win_condition(game).friends_with_conclusion(GameConclusion::Town)
        }
    }
}