
use serde::Serialize;

use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::Game;
use super::{RoleStateImpl};

#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vigilante {
    // Empty struct since all functionality is now in the VigilanteGun ability
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Vigilante {
    type ClientRoleState = Vigilante;
    
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        // Give the player a Vigilante Gun ability when the role is created
        actor_ref.add_ability(game, crate::game::ability::AbilityID::VigilanteGun);
    }
}