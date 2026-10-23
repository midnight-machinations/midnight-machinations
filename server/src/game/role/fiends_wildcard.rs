use serde::{Serialize, Deserialize};
use crate::game::prelude::*;
use super::wild_card::Wildcard;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FiendsWildcard;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for FiendsWildcard {
    type ClientAbilityState = FiendsWildcard;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, _input_player: PlayerReference, ability_input: crate::game::prelude::ControllerInput) {
        if BlockedComponent::blocked(game, actor_ref) {return}
        if actor_ref.ability_deactivated_from_death(game) {return}
        if ability_input.id() != (ControllerID::Role { player: actor_ref, role: Role::FiendsWildcard, id: 0 }) {return}
        Wildcard::become_role(game, actor_ref, Role::FiendsWildcard);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::FiendsWildcard, 0))
            .single_role_selection_typical(game, |role|RoleSet::Fiends.get_roles().contains(role) && *role != Role::FiendsWildcard)
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
            .allow_players([actor_ref])
            .build_map()
    }
}