use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::event::on_ability_creation::OnAbilityCreationPriority;
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
    fn before_initial_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {

        let possible_roles = Self::POSSIBLE_ROLES.into_iter()
            .filter(|role|role_enabled_and_not_taken(
                *role,
                &game.settings,
                PlayerReference::all_players(game).map(|p|p.role(game)).collect::<Vec<_>>().as_slice()
            ))
            .collect::<Vec<_>>();

        //special case here. I don't want to use set_role because it alerts the player their role changed
        //NOTE: It will still send a packet to the player that their role state updated,
        //so it might be deducible that the player is a drunk
        if let Some(random_town_role) = possible_roles.choose(&mut rand::rng()) {
            actor_ref.set_role_state(game, random_town_role.new_state(game));
        }

    }
    fn on_role_switch(game: &mut Game, player: PlayerReference) {
        Confused::remove_player(game, player);
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &crate::game::event::on_ability_creation::OnAbilityCreation, fold: &mut crate::game::event::on_ability_creation::OnAbilityCreationFold, priority: crate::game::event::on_ability_creation::OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || fold.cancelled {return;}
        if let AbilityID::Role { role, player } = event.id && role == Role::Drunk && player == actor_ref {
            Confused::add_player(game, actor_ref);
        }
    }
}
impl Drunk{
    const POSSIBLE_ROLES: [Role; 7] = [
        Role::Detective, Role::Snoop, Role::Gossip,
        Role::Philosopher, Role::Psychic, Role::TallyClerk,
        Role::Auditor
    ];
}