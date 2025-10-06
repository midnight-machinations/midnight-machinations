use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_role_switch::OnRoleSwitch;
use crate::game::game_conclusion::GameConclusion;
use crate::game::role::Role;
use crate::game::role_list::role_enabled_and_not_taken;
use crate::game::{attack_power::DefensePower, role_list::RoleSet};
use crate::game::player::PlayerReference;
use crate::game::Game;

use super::RoleStateTrait;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Pawn;

impl RoleStateTrait for Pawn {
    type ClientAbilityState = Pawn;
    fn on_role_switch(self, game: &mut Game, actor_ref: PlayerReference, event: &OnRoleSwitch, _fold: &mut (), _priority: ()) {
        if actor_ref != event.player {return}
        AbilityID::Role { role: Role::Pawn, player: actor_ref }.delete_ability(game);
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Pawn) || fold.cancelled {return}
        
        let possible_roles = RoleSet::TownInvestigative
            .get_roles()
            .into_iter()
            .filter(|role|role_enabled_and_not_taken(
                *role,
                &game.settings,
                PlayerReference::all_players(game).map(|p|p.role(game)).collect::<Vec<_>>().as_slice()
            ))
            .collect::<Vec<_>>();

        if let Some(new_role) = possible_roles.choose(&mut rand::rng()) {
            actor_ref.set_role_state_without_deleting_previous(game, new_role.new_state(game));

            for player in PlayerReference::all_players(game){
                if
                    !player.win_condition(game).friends_with_conclusion(GameConclusion::Town) &&
                    player != actor_ref
                {
                    player.add_private_chat_message(game, ChatMessageVariant::PawnRole{role: *new_role});
                }
            }
        }
    }
}