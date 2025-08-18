use std::collections::HashSet;

use serde::Serialize;

use crate::game::ability_input::AvailableBooleanSelection;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::detained::Detained;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::game_conclusion::GameConclusion;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::role::BooleanSelection;
use crate::game::Game;

use super::{
    ControllerID, ControllerParametersMap, PlayerListSelection,
    Role, RoleStateImpl
};


#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Jailor { 
    pub jailed_target_ref: Option<PlayerReference>, 
    executions_remaining: u8
}

impl Default for Jailor {
    fn default() -> Self {
        Self { 
            jailed_target_ref: None, 
            executions_remaining: 3
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Jailor {
    type ClientRoleState = Jailor;
    fn new_state(game: &Game) -> Self {
        Self{
            executions_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Kill => {

                let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Jailor, 1).get_boolean_selection(game) else {return};
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
    
                    self.executions_remaining = 
                        if target.win_condition(game).is_loyalist_for(GameConclusion::Town) {0} else {self.executions_remaining.saturating_sub(1)};
                    actor_ref.set_role_state(game, self);
                }
                
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Jailor, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game))
                .reset_on_phase_start(PhaseType::Night)
                .allow_players([actor_ref])
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Jailor, 1))
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
    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, 
            if PlayerReference::all_players(game).any(|p|Detained::is_detained(game, p)) {
                vec![ChatGroup::Jail].into_iter().collect()
            }else{
                vec![]
            }
        )
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        let mut out = crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref);
        if 
            game.current_phase().is_night() &&
            !actor_ref.ability_deactivated_from_death(game) &&
            PlayerReference::all_players(game).any(|p|Detained::is_detained(game, p))
        {
            out.insert(ChatGroup::Jail);
        }
        out
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                let Some(PlayerListSelection(target)) = ControllerID::role(actor_ref, Role::Jailor, 0)
                    .get_player_list_selection(game)
                    .cloned() else {return};
                let Some(target) = target.first() else {return};

                if actor_ref.ability_deactivated_from_death(game) || !target.alive(game) {return};
                
                self.jailed_target_ref = Some(*target);
                
                actor_ref.set_role_state(game, self);

                Detained::add_detain(game, *target);
                actor_ref.add_private_chat_message(game, 
                    ChatMessageVariant::JailedTarget{ player_index: target.index() }
                );
            },
            PhaseType::Obituary => {
                self.jailed_target_ref = None;
                actor_ref.set_role_state(game, self);
            },
            _ => {}
        }
    }
}