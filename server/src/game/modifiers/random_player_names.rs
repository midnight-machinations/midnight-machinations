
use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct RandomPlayerNames;

impl From<&RandomPlayerNames> for ModifierID{
    fn from(_: &RandomPlayerNames) -> Self {
        ModifierID::RandomPlayerNames
    }
}
impl ModifierStateImpl for RandomPlayerNames{}
