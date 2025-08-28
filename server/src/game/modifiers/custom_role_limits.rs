
use serde::{Deserialize, Serialize};

use crate::{game::role::Role, vec_map::VecMap};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CustomRoleLimits {
    pub limits: VecMap<Role, u8>
}

/*
    There is modifier specific code in the set_verdict() function
*/
impl From<&CustomRoleLimits> for ModifierID{
    fn from(_: &CustomRoleLimits) -> Self {
        ModifierID::CustomRoleLimits
    }
}
impl ModifierStateImpl for CustomRoleLimits{}
