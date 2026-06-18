use crate::game::abilities::{pawn_convert::PawnConvert, pitchfork::PitchforkAbility, role_abilities::RoleAbility, syndicate_gun::SyndicateGun};

#[derive(Clone, Debug)]
pub enum Ability{
    Role(RoleAbility),
    Pitchfork(PitchforkAbility),
    SyndicateGun(SyndicateGun),
    PawnConvert(PawnConvert)
}
