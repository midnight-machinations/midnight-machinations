use serde::Serialize;

use crate::game::{
    abilities_component::ability_id::AbilityID, attack_power::DefensePower, components::pitchfork_item::PitchforkItemComponent, event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority}, player::PlayerReference, role::Role, Game
};


use super::{RoleState, RoleStateTrait};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Rabblerouser;

impl RoleStateTrait for Rabblerouser {
    type ClientAbilityState = Rabblerouser;
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || fold.cancelled {return;}
        if let AbilityID::Role{role, player} = event.id && player == actor_ref && role == Role::Rabblerouser {
            PitchforkItemComponent::give_pitchfork(game, actor_ref);
        }
    }
    fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _old: RoleState, _new: RoleState) {
        if player == actor_ref {
            PitchforkItemComponent::remove_pitchfork(game, actor_ref);
        }
    }
}

