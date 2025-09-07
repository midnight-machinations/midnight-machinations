use crate::game::abilities::{pitchfork::PitchforkAbility, role_abilities::RoleAbility, syndicate_gun::SyndicateGun};

#[derive(Clone)]
pub enum Ability{
    RoleAbility(RoleAbility),
    Pitchfork(PitchforkAbility),
    SyndicateGun(SyndicateGun),
}

