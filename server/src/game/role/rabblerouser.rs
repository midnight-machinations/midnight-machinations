use serde::Serialize;

use crate::game::{
    attack_power::DefensePower, components::pitchfork_item::PitchforkItemComponent, event::{on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority}, on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority}}, player::PlayerReference, role::Role, Game
};


use super::RoleStateTrait;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Rabblerouser;

impl RoleStateTrait for Rabblerouser {
    type ClientAbilityState = Rabblerouser;
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Rabblerouser) {return}
        PitchforkItemComponent::give_pitchfork(game, actor_ref);
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Rabblerouser) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        PitchforkItemComponent::remove_pitchfork(game, actor_ref);
    }
}

