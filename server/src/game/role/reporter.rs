use std::collections::HashSet;

use serde::Serialize;

use crate::game::controllers::{AvailableBooleanSelection, AvailableStringSelection};
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::silenced::Silenced;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;


use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap,
    PlayerListSelection, Role, RoleStateImpl
};

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reporter {
    pub interviewed_target: Option<PlayerReference>, 
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Reporter {
    type ClientRoleState = Reporter;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if 
            priority == OnMidnightPriority::Investigative &&
            Self::get_public(game, actor_ref) && 
            !actor_ref.ability_deactivated_from_death(game) &&
            !actor_ref.night_blocked(midnight_variables) &&
            !Silenced::silenced(game, actor_ref)
        {
            game.add_message_to_chat_group(
                ChatGroup::All, 
                ChatMessageVariant::ReporterReport { report: Self::get_report(game, actor_ref)}
            );    
        }
    }
    
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reporter, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .reset_on_phase_start(PhaseType::Night)
                .allow_players([actor_ref])
                .build_map(),
            // Publish
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reporter, 1))
                .available_selection(AvailableBooleanSelection)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map(),
            // Report
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Reporter, 2))
                .available_selection(AvailableStringSelection)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .allow_players([actor_ref])
                .build_map()
        ])
    }
    fn get_current_send_chat_groups(self,  game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, 
            if 
                game.current_phase().is_night() &&
                !actor_ref.ability_deactivated_from_death(game) &&
                self.interviewed_target.map_or_else(||false, |p|p.alive(game))
            {
                vec![ChatGroup::Interview]
            }else{
                vec![]
            }
        )
    }
    fn get_current_receive_chat_groups(self,  game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        let mut out = crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref);
        if 
            game.current_phase().is_night() &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.interviewed_target.map_or_else(||false, |p|p.alive(game))
        {
            out.insert(ChatGroup::Interview);
        }
        out
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        match phase {
            PhaseType::Night => {
                let Some(PlayerListSelection(target)) = ControllerID::role(actor_ref, Role::Reporter, 0)
                    .get_player_list_selection(game)
                    else {return};
                let Some(target) = target.first().copied() else {return};

                if actor_ref.ability_deactivated_from_death(game) || !target.alive(game) {return};
                
                self.interviewed_target = Some(target);
                
                actor_ref.set_role_state(game, self);

                InsiderGroupID::send_message_in_available_insider_chat_or_private(
                    game,
                    target,
                    ChatMessageVariant::PlayerIsBeingInterviewed { player_index: target.index() },
                    true
                );
            },
            PhaseType::Obituary => {
                self.interviewed_target = None;
                actor_ref.set_role_state(game, self);
            },
            _ => {}
        }
    }
}

impl Reporter{
    fn get_report(game: &Game, actor_ref: PlayerReference)->String{
        ControllerID::role(actor_ref, Role::Reporter, 2).get_string_selection(game).map_or_else(String::new, |s|s.0.clone())
    }
    fn get_public(game: &Game, actor_ref: PlayerReference)->bool{
        ControllerID::role(actor_ref, Role::Reporter, 1).get_boolean_selection(game).is_some_and(|b|b.0)
    }
}