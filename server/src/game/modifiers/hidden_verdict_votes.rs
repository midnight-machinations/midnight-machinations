
use serde::{Deserialize, Serialize};

use crate::game::{abilities_component::ability_id::AbilityID, components::blocked::BlockedComponent, player::PlayerReference, role::Role, Game};

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
        PlayerReference::all_players(game)
            .filter(|p|!p.ability_deactivated_from_death(game))
            .any(|p|
                !BlockedComponent::blocked(game, p) && (
                    AbilityID::Role { role: Role::Blackmailer, player: p }.exists(game) ||
                    AbilityID::Role { role: Role::Cerenovous, player: p }.exists(game)
                )
            )
    }
}
