use crate::game::abilities::role_abilities::RoleAbility;

#[derive(Clone)]
pub enum Ability{
    RoleAbility(RoleAbility),
}

