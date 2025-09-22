use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoMajority;

impl From<&NoMajority> for ModifierID{
    fn from(_: &NoMajority) -> Self {
        ModifierID::NoMajority
    }
}

impl ModifierStateImpl for NoMajority {}
