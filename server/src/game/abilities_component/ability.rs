use crate::game::abilities::{pitchfork::PitchforkAbility, role_abilities::RoleAbility};

#[derive(Clone)]
pub enum Ability{
    RoleAbility(RoleAbility),
    Pitchfork(PitchforkAbility),
}

