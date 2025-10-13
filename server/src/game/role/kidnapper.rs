use serde::Serialize;
use crate::game::controllers::{AvailableBooleanSelection, BooleanSelection};
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap};
use crate::game::components::detained::Detained;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::Game;
use crate::game::abilities_component::ability_id::AbilityID;

use super::{
    ControllerID,
    ControllerParametersMap, Role,
    RoleStateTrait
};


#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Kidnapper { 
    pub jailed_target_ref: Option<PlayerReference>, 
    executions_remaining: u8
}

impl Default for Kidnapper {
    fn default() -> Self {
        Self { 
            jailed_target_ref: None, 
            executions_remaining: 1
        }
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Kidnapper {
    type ClientAbilityState = Kidnapper;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {


        match priority {
            OnMidnightPriority::Kill => {
                let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Kidnapper, 1).get_boolean_selection(game) else {return};
                let Some(target) = self.jailed_target_ref else {return};
                
                if Detained::is_detained(game, target){
                    target.try_night_kill_single_attacker(
                        actor_ref, 
                        game, 
                        midnight_variables,
                        GraveKiller::Role(Role::Jailor),
                        AttackPower::ProtectionPiercing, 
                        false
                    );
    
                    self.executions_remaining = self.executions_remaining.saturating_sub(1);
                    actor_ref.edit_role_ability_helper(game, self);
                }
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Kidnapper, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .reset_on_phase_start(PhaseType::Night)
                .allow_players([actor_ref])
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Kidnapper, 1))
                .available_selection(AvailableBooleanSelection)
                .night_typical(actor_ref)
                .add_grayed_out_condition(
                    self.executions_remaining == 0 ||
                    game.day_number() <= 1 ||
                    self.jailed_target_ref.is_none()
                )
                .build_map()
        ])
    }
    fn send_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            game.current_phase().phase() == PhaseType::Night &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.jailed_target_ref.is_some()
        {
            out.insert(actor_ref, ChatGroup::Kidnapped);
        }
        if let Some(target) = self.jailed_target_ref && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Kidnapped);
        }
        
        out
    }
    fn receive_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference)-> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            game.current_phase().phase() == PhaseType::Night &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.jailed_target_ref.is_some()
        {
            out.insert(actor_ref, ChatGroup::Kidnapped);
        }
        if let Some(target) = self.jailed_target_ref && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Kidnapped);
        }
        
        out
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                let Some(target) = ControllerID::role(actor_ref, Role::Kidnapper, 0)
                    .get_player_list_selection(game)
                    .and_then(|p|p.0.first())
                    .copied()
                else {return};

                if actor_ref.ability_deactivated_from_death(game) || !target.alive(game) {return};
                
                self.jailed_target_ref = Some(target);
                
                actor_ref.edit_role_ability_helper(game, self);

                Detained::add_detain(game, target);
                actor_ref.add_private_chat_message(game, 
                    ChatMessageVariant::JailedTarget{ player_index: target }
                );
            },
            PhaseType::Obituary => {
                self.jailed_target_ref = None;
                actor_ref.edit_role_ability_helper(game, self);
            },
            _ => {}
        }
    }
}