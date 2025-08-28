use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct DeadCanChat;

/*
    There is modifier specific code in the common_role::get_send_chat_groups() function
*/
impl From<&DeadCanChat> for ModifierID{
    fn from(_: &DeadCanChat) -> Self {
        ModifierID::DeadCanChat
    }
}

impl ModifierStateImpl for DeadCanChat{}