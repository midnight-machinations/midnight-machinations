
use crate::game::{components::silenced::Silenced, modifiers::Modifiers, Game};

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
        !Silenced::is_empty(game)
    }
}