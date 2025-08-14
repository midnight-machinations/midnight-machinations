
use crate::game::{modifiers::Modifiers, player::PlayerReference, role::Role, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct HiddenNominationVotes;

impl From<&HiddenNominationVotes> for ModifierType{
    fn from(_: &HiddenNominationVotes) -> Self {
        ModifierType::HiddenNominationVotes
    }
}
impl ModifierTrait for HiddenNominationVotes{}

impl HiddenNominationVotes {
    pub fn nomination_votes_are_hidden(game: &Game)->bool{
        Modifiers::is_enabled(game, ModifierType::HiddenNominationVotes) ||
        PlayerReference::all_players(game)
            .filter(|p|p.alive(game))
            .any(|p|matches!(p.role(game), Role::Blackmailer | Role::Cerenovous))
    }
}