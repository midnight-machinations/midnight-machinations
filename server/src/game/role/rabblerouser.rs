use serde::Serialize;

use crate::game::{
    attack_power::DefensePower,
    components::pitchfork_item::PitchforkItemComponent,
    player::PlayerReference, 
    Game
};


use super::{RoleState, RoleStateTrait};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Rabblerouser;

impl RoleStateTrait for Rabblerouser {
    type ClientAbilityState = Rabblerouser;
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        PitchforkItemComponent::give_pitchfork(game, actor_ref);
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _old: RoleState, _new: RoleState) {
        if player == actor_ref {
            PitchforkItemComponent::remove_pitchfork(game, actor_ref);
        }
    }
}

