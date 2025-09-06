use serde::Serialize;

use crate::game::attack_power::DefensePower;


use super::RoleStateTrait;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Villager;

pub type ClientRoleState = Villager;

impl RoleStateTrait for Villager {
    type ClientAbilityState = Villager;
}

