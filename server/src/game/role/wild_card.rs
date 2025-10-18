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
pub struct Wildcard;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Wildcard {
    type ClientAbilityState = Wildcard;
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match phase {
            PhaseType::Dusk => {
                if actor_ref.ability_deactivated_from_death(game) {return;}
                Wildcard::become_role(game, actor_ref, Role::Wildcard);
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Wildcard, 0))
            .single_role_selection_typical(game, |role|*role != Role::Wildcard)
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
            .allow_players([actor_ref])
            .build_map()
    }
}

impl Wildcard {
    pub fn become_role(game: &mut Game, actor_ref: PlayerReference, role: Role) {

        let Some(&role) = ControllerID::role(actor_ref, role, 0)
            .get_role_list_selection_first(game)
            else {return};

        if 
            role_enabled_and_not_taken(
                role, 
                &game.settings, 
                &PlayerReference::all_players(game)
                    .map(|player_ref| player_ref.role(game))
                    .collect::<Vec<Role>>()
            )
        {
            let new_state = role.new_state(game);
            actor_ref.set_role_win_con_insider_group(game, new_state);
        }else{
            actor_ref.add_private_chat_message(game, ChatMessageVariant::WildcardConvertFailed{role})
        }
    }
}