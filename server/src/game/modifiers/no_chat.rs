use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct NoChat;

impl From<&NoChat> for ModifierID{
    fn from(_: &NoChat) -> Self {
        ModifierID::NoChat
    }
}

impl ModifierStateImpl for NoChat {}
