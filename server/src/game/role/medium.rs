use serde::Serialize;
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap};
use crate::game::components::detained::Detained;
use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::role::common_role;
use crate::game::Game;

use super::{
    ControllerID, ControllerParametersMap, Role, RoleStateTrait
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium{
    pub haunts_remaining: u8,
    pub haunted_target: Option<PlayerReference>,
    pub seanced_target: Option<PlayerReference>,
}

impl Default for Medium{
    fn default() -> Self {
        Self { haunts_remaining: 3, haunted_target: None, seanced_target: None}
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Medium {
    type ClientAbilityState = Medium;
    fn new_state(game: &Game) -> Self {
        Self{
            haunts_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        //seance
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Medium, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|target_ref|
                        !target_ref.alive(game) &&
                        actor_ref != *target_ref
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
            .reset_on_phase_start(PhaseType::Night)
            .allow_players([actor_ref])
            .build_map(),
        //haunt
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Medium, 1))
            .single_player_selection_typical(actor_ref, false, true)
            .add_grayed_out_condition(actor_ref.alive(game) || self.haunts_remaining == 0)
            .reset_on_phase_start(PhaseType::Night)
            .allow_players([actor_ref])
            .build_map()
        ])
    }
    fn send_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            !actor_ref.ability_deactivated_from_death(game) &&
            (
                (
                    !Detained::is_detained(game, actor_ref) &&
                    game.current_phase().phase() == PhaseType::Night
                ) || 
                game.current_phase().phase() == PhaseType::Obituary
            )
        {
            out.insert(actor_ref, ChatGroup::Dead);
        }
        if let Some(target) = self.haunted_target && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Dead);
        }
        out
    }
    fn receive_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference)->crate::game::chat::PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            !actor_ref.ability_deactivated_from_death(game) &&
            (
                (
                    !Detained::is_detained(game, actor_ref) &&
                    game.current_phase().phase() == PhaseType::Night
                ) || 
                game.current_phase().phase() == PhaseType::Obituary
            )
        {
            out.insert(actor_ref, ChatGroup::Dead);
        }
        if let Some(target) = self.haunted_target && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Dead);
        }
        out
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Discussion => {
                self.haunted_target = None;
                actor_ref.edit_role_ability_helper(game, self);
            },
            PhaseType::Night => {
                if let Some(target) = ControllerID::role(actor_ref, Role::Medium, 1)
                    .get_player_list_selection(game)
                    .and_then(|p|p.0.first())
                    .copied()
                {
                    self.haunts_remaining = self.haunts_remaining.saturating_sub(1);
                    self.haunted_target = Some(target);
                    //need to run set_role_state first because otherwise the haunted player will not get the message
                    actor_ref.edit_role_ability_helper(game, self.clone());

                    game.add_message_to_chat_group(ChatGroup::Dead,
                        ChatMessageVariant::MediumHauntStarted {
                            medium: actor_ref,
                            player: target
                        }
                    );
                };

                //reset old
                self.seanced_target = None;
                
                //set new
                if let Some(target) = ControllerID::role(actor_ref, Role::Medium, 0)
                    .get_player_list_selection(game)
                    .and_then(|p|p.0.first())
                    .copied() && 
                    !actor_ref.ability_deactivated_from_death(game)
                {
                    self.seanced_target = Some(target);
                    game.add_message_to_chat_group(ChatGroup::Dead,
                        ChatMessageVariant::MediumSeance {
                            medium: actor_ref,
                            player: target
                        }
                    );
                };
                
                actor_ref.edit_role_ability_helper(game, self);
            },
            _=>{}
        }
    }
    fn on_player_roleblocked(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, invisible: bool) {
        common_role::on_player_roleblocked(midnight_variables, actor_ref, player);
        if player != actor_ref {return}
        if let Some(seanced) = self.seanced_target {
            seanced.roleblock(game, midnight_variables, invisible);
        }
    }
}
