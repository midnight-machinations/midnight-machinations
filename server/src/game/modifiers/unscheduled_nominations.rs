use super::{ModifierTrait, ModifierType};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct UnscheduledNominations;

/*
    There is modifier specific code in the common_role::get_send_chat_groups() function
*/
impl From<&UnscheduledNominations> for ModifierType{
    fn from(_: &UnscheduledNominations) -> Self {
        ModifierType::UnscheduledNominations
    }
}

impl ModifierTrait for UnscheduledNominations{}