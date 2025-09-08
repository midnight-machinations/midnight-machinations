use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::chat::ChatMessageVariant;
use crate::game::game_conclusion::GameConclusion;
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
    fn before_initial_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {

        let possible_roles = RoleSet::TownInvestigative
            .get_roles()
            .into_iter()
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

            for player in PlayerReference::all_players(game){
                if
                    !player.win_condition(game).friends_with_conclusion(GameConclusion::Town) &&
                    player != actor_ref
                {
                    player.add_private_chat_message(game, ChatMessageVariant::PawnRole{role: *random_town_role});
                }
            }
        }

    }
}