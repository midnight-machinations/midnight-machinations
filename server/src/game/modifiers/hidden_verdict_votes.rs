
use crate::game::{modifiers::Modifiers, player::PlayerReference, role::Role, Game};

use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct HiddenVerdictVotes;

impl From<&HiddenVerdictVotes> for ModifierType{
    fn from(_: &HiddenVerdictVotes) -> Self {
        ModifierType::HiddenVerdictVotes
    }
}
impl ModifierTrait for HiddenVerdictVotes{}

impl HiddenVerdictVotes {
    pub fn verdict_votes_are_hidden(game: &Game)->bool{
        Modifiers::is_enabled(game, ModifierType::HiddenVerdictVotes) ||
        PlayerReference::all_players(game).any(|p|
            p.role(game) == Role::Blackmailer || p.alive(game)
        )
    }
}
