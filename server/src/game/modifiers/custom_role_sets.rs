
use serde::{Deserialize, Serialize};

use crate::{game::role::Role, vec_set::VecSet};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CustomRoleSets {
    pub sets: Vec<CustomRoleSet>
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CustomRoleSet {
    pub name: String,
    pub roles: VecSet<Role>
}

impl From<&CustomRoleSets> for ModifierID{
    fn from(_: &CustomRoleSets) -> Self {
        ModifierID::CustomRoleSets
    }
}
impl ModifierStateImpl for CustomRoleSets{}
