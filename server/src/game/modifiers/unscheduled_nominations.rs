use serde::{Deserialize, Serialize};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct UnscheduledNominations;

/*
    There is modifier specific code in the common_role::get_send_chat_groups() function
*/
impl From<&UnscheduledNominations> for ModifierID{
    fn from(_: &UnscheduledNominations) -> Self {
        ModifierID::UnscheduledNominations
    }
}

impl ModifierStateImpl for UnscheduledNominations{}