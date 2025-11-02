
use serde::{Deserialize, Serialize};

use crate::game::{abilities_component::{ability_id::AbilityID, Abilities}, components::blocked::BlockedComponent, role::Role, Game};

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
                let AbilityID::Role { player, role } = id else {return false};
                if BlockedComponent::blocked(game, player) {return false}
                if player.ability_deactivated_from_death(game) {return false}
                role == Role::Cerenovous || role == Role::Blackmailer
            })
    }
}