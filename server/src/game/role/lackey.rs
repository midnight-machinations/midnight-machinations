use serde::Serialize;
use crate::game::attack_power::DefensePower;
use super::RoleStateTrait;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Lackey;

pub type ClientRoleState = Lackey;

impl RoleStateTrait for Lackey {
    type ClientAbilityState = Lackey;
}
