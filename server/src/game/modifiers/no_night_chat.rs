use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoNightChat;

impl From<&NoNightChat> for ModifierID{
    fn from(_: &NoNightChat) -> Self {
        ModifierID::NoNightChat
    }
}

impl ModifierStateImpl for NoNightChat {}
