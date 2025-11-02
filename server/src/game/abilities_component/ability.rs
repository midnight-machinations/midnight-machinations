use crate::game::abilities::{pitchfork::PitchforkAbility, role_abilities::RoleAbility, syndicate_gun::SyndicateGun};

#[derive(Clone, Debug)]
pub enum Ability{
    Role(RoleAbility),
    Pitchfork(PitchforkAbility),
    SyndicateGun(SyndicateGun),
}
