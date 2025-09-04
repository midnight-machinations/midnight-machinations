use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ForfeitNominationVote;

impl From<&ForfeitNominationVote> for ModifierID{
    fn from(_: &ForfeitNominationVote) -> Self {
        ModifierID::ForfeitNominationVote
    }
}
impl ModifierStateImpl for ForfeitNominationVote{}
