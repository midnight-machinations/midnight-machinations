use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::event::on_ability_creation::OnAbilityCreationPriority;
use crate::game::event::on_ability_deletion::OnAbilityDeletionPriority;
use crate::game::role_list::role_enabled_and_not_taken;
use crate::game::{attack_power::DefensePower, components::confused::Confused};
use crate::game::player::PlayerReference;
use crate::game::Game;

use super::{Role, RoleStateTrait};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Drunk;

impl RoleStateTrait for Drunk {
    type ClientAbilityState = Drunk;
    fn on_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {
        if actor_ref != player {return}
        AbilityID::Role { role: Role::Drunk, player: actor_ref }.delete_ability(game);
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Drunk) || fold.cancelled {return}
        
        Confused::add_player(game, actor_ref);
        let possible_roles = Self::POSSIBLE_ROLES.into_iter()
            .filter(|role|role_enabled_and_not_taken(
                *role,
                &game.settings,
                PlayerReference::all_players(game).map(|p|p.role(game)).collect::<Vec<_>>().as_slice()
            ))
            .collect::<Vec<_>>();

        if let Some(new_role) = possible_roles.choose(&mut rand::rng()) {
            actor_ref.set_role_state_without_deleting_previous(game, new_role.new_state(game));
        }
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_deletion::OnAbilityDeletion, _fold: &mut (), priority: crate::game::event::on_ability_deletion::OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Drunk) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Confused::remove_player(game, actor_ref);
    }
}
impl Drunk{
    const POSSIBLE_ROLES: [Role; 7] = [
        Role::Detective, Role::Snoop, Role::Gossip,
        Role::Philosopher, Role::Psychic, Role::TallyClerk,
        Role::Auditor
    ];
}