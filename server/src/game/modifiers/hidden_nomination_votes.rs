
use serde::{Deserialize, Serialize};

use crate::game::{abilities_component::ability_id::AbilityID, components::blocked::BlockedComponent, player::PlayerReference, role::Role, Game};

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
        PlayerReference::all_players(game)
            .filter(|p|p.alive(game))
            .any(|p|
                !BlockedComponent::blocked(game, p) && (
                    AbilityID::Role { role: Role::Blackmailer, player: p }.exists(game) ||
                    AbilityID::Role { role: Role::Cerenovous, player: p }.exists(game)
                )
            )
    }
}