use serde::Serialize;
use crate::game::ability_input::AvailablePlayerListSelection;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::phase::PhaseType;
use crate::game::{attack_power::DefensePower, player::PlayerReference};

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Cerenovous{
    pub currently_piloted: Option<PlayerReference>,
    previous: Option<PlayerReference>,
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Cerenovous {
    type ClientRoleState = Cerenovous;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}
        

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){
            let target_ref = visit.target;
    
            self.currently_piloted = Some(target_ref);
            self.previous = Some(target_ref);
        }else{
            self.previous = None;
        }
        actor_ref.set_role_state(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Cerenovous, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|player|
                        !(!player.alive(game) || 
                        *player == actor_ref ||
                        InsiderGroupID::in_same_group(game, actor_ref, *player) ||
                        Some(*player) == self.previous)
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Cerenovous, 0),
            false
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: crate::game::phase::PhaseType) {
        self.currently_piloted = None;
        if phase == PhaseType::Night {
            actor_ref.set_role_state(game, self);
        }
    }
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if priority == WhisperPriority::Send && !fold.cancelled && event.receiver != actor_ref && event.sender != actor_ref {
            actor_ref.add_private_chat_message(game, ChatMessageVariant::Whisper {
                from_player_index: event.sender.into(),
                to_player_index: event.receiver.into(),
                text: event.message.clone()
            });
        }
    }
}
