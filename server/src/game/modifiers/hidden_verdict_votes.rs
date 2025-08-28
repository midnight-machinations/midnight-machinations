
use serde::{Deserialize, Serialize};

use crate::game::{player::PlayerReference, role::Role, Game};

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
            .filter(|p|p.alive(game))
            .any(|p|matches!(p.role(game), Role::Blackmailer | Role::Cerenovous))
    }
}
