use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct TwoThirdsMajority;

impl From<&TwoThirdsMajority> for ModifierID{
    fn from(_: &TwoThirdsMajority) -> Self {
        ModifierID::TwoThirdsMajority
    }
}

impl ModifierStateImpl for TwoThirdsMajority {}
