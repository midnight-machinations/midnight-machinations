
use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct RandomPlayerNames;

impl From<&RandomPlayerNames> for ModifierType{
    fn from(_: &RandomPlayerNames) -> Self {
        ModifierType::RandomPlayerNames
    }
}
impl ModifierTrait for RandomPlayerNames{}
