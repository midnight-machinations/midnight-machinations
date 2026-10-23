
use serde::{Deserialize, Serialize};

use crate::game::{Game, abilities_component::{Abilities, ability_id::AbilityID}, components::blocked::BlockedComponent, controllers::ControllerID, role::Role};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct HiddenNominationVotes;

impl From<&HiddenNominationVotes> for ModifierID{
    fn from(_: &HiddenNominationVotes) -> Self {
        ModifierID::HiddenNominationVotes
    }
}
impl ModifierStateImpl for HiddenNominationVotes{}

impl HiddenNominationVotes {
    pub fn nomination_votes_are_hidden(game: &Game)->bool{
        game.modifier_settings().is_enabled(ModifierID::HiddenNominationVotes) ||
        Abilities::ids(game)
            .into_iter()
            .any(|id|{
                let AbilityID::Role { player, role: _ } = id else {return false};
                if BlockedComponent::blocked(game, player) {return false}
                if player.ability_deactivated_from_death(game) {return false}
                ControllerID::role(player, Role::Blackmailer, 1).get_boolean_selection(game).map(|o|o.0).unwrap_or(false) ||
                ControllerID::role(player, Role::Pyrotechnician, 1).get_boolean_selection(game).map(|o|o.0).unwrap_or(false) ||
                ControllerID::role(player, Role::Cerenovous, 1).get_boolean_selection(game).map(|o|o.0).unwrap_or(false)
            })
    }
}