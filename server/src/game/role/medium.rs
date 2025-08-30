use std::collections::HashSet;

use serde::Serialize;

use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::detained::Detained;
use crate::game::controllers::AvailablePlayerListSelection;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;

use super::{
    ControllerID, ControllerParametersMap, Role, RoleStateImpl
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

impl RoleStateImpl for Medium {
    type ClientRoleState = Medium;
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
    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        let mut out = crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![ChatGroup::Dead]);

        if 
            (game.current_phase().phase() == PhaseType::Obituary) &&
            actor_ref.alive(game)
        {
            out.insert(ChatGroup::Dead);
        }
        out
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        let mut out = crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref);

        if 
            (
                (
                    !Detained::is_detained(game, actor_ref) &&
                    game.current_phase().phase() == PhaseType::Night
                ) || 
                game.current_phase().phase() == PhaseType::Obituary
            ) &&
            actor_ref.alive(game)
        {
            out.insert(ChatGroup::Dead);
        }
        out
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Discussion => {
                self.haunted_target = None;
                actor_ref.set_role_state(game, self);
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
                    actor_ref.set_role_state(game, self.clone());

                    game.add_message_to_chat_group(ChatGroup::Dead,
                        ChatMessageVariant::MediumHauntStarted {
                            medium: actor_ref.index(),
                            player: target.index()
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
                            medium: actor_ref.index(),
                            player: target.index()
                        }
                    );
                };
                
                actor_ref.set_role_state(game, self);
            },
            _=>{}
        }
    }
}
