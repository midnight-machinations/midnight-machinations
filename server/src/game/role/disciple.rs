use serde::Serialize;
use crate::game::attack_power::DefensePower;
use super::RoleStateTrait;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Disciple;

pub type ClientRoleState = Disciple;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Disciple {
    type ClientAbilityState = Disciple;
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Cult
        ].into_iter().collect()
    }
}