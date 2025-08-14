use std::collections::HashSet;

use serde::Serialize;

use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::detained::Detained;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;

use super::{
    ControllerID, ControllerParametersMap, PlayerListSelection, Role, RoleStateImpl
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Medium{
    pub seances_remaining: u8,
    pub seanced_target: Option<PlayerReference>
}

impl Default for Medium{
    fn default() -> Self {
        Self { seances_remaining: 3, seanced_target: None}
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Medium {
    type ClientRoleState = Medium;
    fn new_state(game: &Game) -> Self {
        Self{
            seances_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Medium, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .add_grayed_out_condition(actor_ref.alive(game) || self.seances_remaining == 0)
            .reset_on_phase_start(PhaseType::Night)
            .allow_players([actor_ref])
            .build_map()
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
                self.seanced_target = None;
                actor_ref.set_role_state(game, self);
            },
            PhaseType::Night => {
                let Some(PlayerListSelection(target)) = ControllerID::role(actor_ref, Role::Medium, 0)
                    .get_player_list_selection(game) else {return};

                let Some(target) = target.first().copied() else {return};
                
                self.seances_remaining = self.seances_remaining.saturating_sub(1);
                self.seanced_target = Some(target);
                
                actor_ref.set_role_state(game, self);

                game.add_message_to_chat_group(ChatGroup::Dead,
                    ChatMessageVariant::MediumHauntStarted{
                        medium: actor_ref.index(),
                        player: target.index()
                    }
                );
            },
            _=>{}
        }
    }
}
