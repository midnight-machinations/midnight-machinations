use serde::{Deserialize, Serialize};

use crate::{game::{components::{insider_group::InsiderGroupID, win_condition::WinCondition}, role::Role, role_list_generation::criteria::GenerationCriterion}, vec_set::VecSet};

#[derive(Hash, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Template{
    Role{role: Role},
    Pawn,
    Drunk
}
impl From<Role> for Template {
    fn from(value: Role) -> Self {
        Self::Role { role: value }
    }

}impl From<&Role> for Template {
    fn from(value: &Role) -> Self {
        Self::Role { role: *value }
    }
}
impl Template{
    pub fn role_list_generation_criteria(self) -> Vec<GenerationCriterion> {
        match self {
            Template::Role { role } => role.role_list_generation_criteria(),
            Template::Pawn => Role::Witch.role_list_generation_criteria(),
            Template::Drunk => Role::Detective.role_list_generation_criteria(),
        }
    }
    pub fn default_insider_groups(self) -> VecSet<InsiderGroupID> {
        match self {
            Template::Role { role } => role.default_state().default_insider_groups(),
            Template::Pawn => VecSet::new(),
            Template::Drunk => VecSet::new(),
        }
    }
    pub fn default_win_condition(self) -> WinCondition {
        match self {
            Template::Role { role } => role.default_state().default_win_condition(),
            Template::Pawn => Role::Witch.default_state().default_win_condition(),
            Template::Drunk => Role::Detective.default_state().default_win_condition(),
        }
    }
    pub fn values()->VecSet<Self>{
        Role::values()
            .into_iter()
            .map(|r|r.into())
            .chain([
                Self::Drunk,
                Self::Pawn
            ])
            .collect()
    }
    pub fn get_role(self)->Option<Role>{
        if let Self::Role { role } = self{
            Some(role)
        }else{
            None
        }
    }
}
impl PartialEq<Template> for Role{
    fn eq(&self, other: &Template) -> bool {
        let temp: Template = self.into();
        temp == *other
    }
}

impl PartialEq<Role> for Template{
    fn eq(&self, other: &Role) -> bool {
        let temp: Template = other.into();
        temp == *self
    }
}