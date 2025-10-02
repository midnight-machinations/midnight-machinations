use rand::prelude::SliceRandom;
use serde::Serialize;
use crate::game::components::blocked::BlockedComponent;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Informant;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Informant {
    type ClientAbilityState = Informant;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return}
        

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        for visit in actor_visits{
            let target_ref = visit.target;

            let mut visited_by: Vec<PlayerReference> = visit.target.lookout_seen_players(midnight_variables, visit).collect();
            visited_by.shuffle(&mut rand::rng());

            let mut visited: Vec<PlayerReference> = target_ref.tracker_seen_players(midnight_variables).collect();
            visited.shuffle(&mut rand::rng());

            let message = ChatMessageVariant::InformantResult{
                player: target_ref,
                role: target_ref.role(game), 
                visited_by,
                visited
            };
            actor_ref.push_night_message(midnight_variables, message);
            actor_ref.reveal_players_role(game, target_ref);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Informant, 0))
            .player_list_selection_typical(actor_ref, false, false, false, false, true, Some(2))
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Informant, 0),
            false
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        Informant::read_whispers(game, actor_ref, event, fold, priority);
    }
}

impl Informant {
    pub fn read_whispers(game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if !BlockedComponent::blocked(game, actor_ref) && priority == WhisperPriority::Send && !fold.cancelled && event.receiver != actor_ref && event.sender != actor_ref {
            actor_ref.add_private_chat_message(game, ChatMessageVariant::Whisper {
                from_player_index: event.sender,
                to_player_index: event.receiver,
                text: event.message.clone()
            });
        }
    }
}