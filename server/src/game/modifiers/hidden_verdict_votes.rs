
use serde::{Deserialize, Serialize};

use crate::game::{abilities_component::{ability_id::AbilityID, Abilities}, components::blocked::BlockedComponent, role::Role, Game};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct HiddenVerdictVotes;

impl From<&HiddenVerdictVotes> for ModifierID{
    fn from(_: &HiddenVerdictVotes) -> Self {
        ModifierID::HiddenVerdictVotes
    }
}
impl ModifierStateImpl for HiddenVerdictVotes{}

impl HiddenVerdictVotes {
    pub fn verdict_votes_are_hidden(game: &Game)->bool{
        game.modifier_settings().is_enabled(ModifierID::HiddenVerdictVotes) ||
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
