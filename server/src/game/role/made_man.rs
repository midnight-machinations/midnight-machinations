use serde::Serialize;
use crate::game::attack_power::DefensePower;
use super::RoleStateTrait;


#[derive(Debug, Clone, Serialize, Default)]
pub struct Goon;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Goon {
    type ClientAbilityState = Goon;
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}