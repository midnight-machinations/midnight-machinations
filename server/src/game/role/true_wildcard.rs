use serde::{Serialize, Deserialize};

use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role_list::role_enabled_and_not_taken;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrueWildcard;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for TrueWildcard {
    type ClientAbilityState = TrueWildcard;
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if phase == PhaseType::Dusk {
            if actor_ref.ability_deactivated_from_death(game) {return;}
            self.become_role(game, actor_ref);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::TrueWildcard, 0))
            .single_role_selection_typical(game, |role|*role != Role::TrueWildcard)
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
            .allow_players([actor_ref])
            .build_map()
    }
}

impl TrueWildcard {
    fn become_role(&self, game: &mut Game, actor_ref: PlayerReference) {

        let Some(&role) = ControllerID::role(actor_ref, Role::TrueWildcard, 0)
            .get_role_list_selection_first(game) else {return};

        if 
            role_enabled_and_not_taken(
                role, 
                &game.settings, 
                &Vec::new(),    //True wildcard can be whatever they want
            )
        {
            let new_state = role.new_state(game);
            actor_ref.set_role_win_con_insider_group(game, new_state);
        }else{
            actor_ref.add_private_chat_message(game, ChatMessageVariant::WildcardConvertFailed{role})
        }
    }
}