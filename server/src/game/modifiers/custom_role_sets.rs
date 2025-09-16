
use serde::{Deserialize, Serialize};

use crate::{game::{role::Role, role_list::RoleSet, settings::Settings}, vec_set::VecSet};

use super::{ModifierStateImpl, ModifierID};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CustomRoleSets {
    pub sets: Vec<CustomRoleSet>
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomRoleSet {
    name: String,
    #[serde(skip_serializing_if = "VecSet::is_empty", default)]
    role_sets: VecSet<CustomRoleSetSubRoleSet>,
    #[serde(skip_serializing_if = "VecSet::is_empty", default)]
    roles: VecSet<Role>
}

impl CustomRoleSet {
    pub fn roles(&self, settings: &Settings) -> VecSet<Role> {
        let mut roles = self.role_sets.iter().fold(VecSet::new(), |mut acc, sub| {
            let role_set_roles = sub.role_set.get_roles(settings);
            acc.extend(role_set_roles.into_iter().filter(|role|!sub.excluded_roles.contains(role)));
            acc
        });
        roles.extend(self.roles.iter().copied());

        roles
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomRoleSetSubRoleSet {
    role_set: RoleSet,
    excluded_roles: VecSet<Role>
}

impl From<&CustomRoleSets> for ModifierID{
    fn from(_: &CustomRoleSets) -> Self {
        ModifierID::CustomRoleSets
    }
}
impl ModifierStateImpl for CustomRoleSets{}
