use serde::Serialize;
use crate::game::controllers::{AvailableBooleanSelection, AvailableStringSelection};
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap};
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::silenced::Silenced;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::abilities_component::ability_id::AbilityID;


use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap,
    PlayerListSelection, Role, RoleStateTrait
};

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Reporter {
    pub interviewed_target: Option<PlayerReference>, 
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Reporter {
    type ClientAbilityState = Reporter;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
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
    fn send_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference)-> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            game.current_phase().phase() == PhaseType::Night &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.interviewed_target.is_some()
        {
            out.insert(actor_ref, ChatGroup::Interview);
        }
        if let Some(target) = self.interviewed_target && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Interview);
        }
        
        out
    }
    fn receive_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference)-> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            game.current_phase().phase() == PhaseType::Night &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.interviewed_target.is_some()
        {
            out.insert(actor_ref, ChatGroup::Interview);
        }
        if let Some(target) = self.interviewed_target && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Interview);
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
                
                actor_ref.edit_role_ability_helper(game, self);

                InsiderGroupID::send_message_in_available_insider_chat_or_private(
                    game,
                    target,
                    ChatMessageVariant::PlayerIsBeingInterviewed { player_index: target },
                    true
                );
            },
            PhaseType::Obituary => {
                self.interviewed_target = None;
                actor_ref.edit_role_ability_helper(game, self);
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