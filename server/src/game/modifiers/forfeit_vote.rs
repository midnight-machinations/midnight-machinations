use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct ForfeitNominationVote;

impl From<&ForfeitNominationVote> for ModifierType{
    fn from(_: &ForfeitNominationVote) -> Self {
        ModifierType::ForfeitNominationVote
    }
}
impl ModifierTrait for ForfeitNominationVote{}
