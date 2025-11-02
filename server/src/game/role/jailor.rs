use serde::Serialize;
use crate::game::prelude::*;


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

impl RoleStateTrait for Jailor {
    type ClientAbilityState = Jailor;
    fn new_state(game: &mut Game) -> Self {
        Self{
            executions_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return}
        let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Jailor, 1).get_boolean_selection(game) else {return};
        let Some(target) = self.jailed_target_ref else {return};
        if !Detained::is_detained(game, target){return}
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
        actor_ref.edit_role_ability_helper(game, self);
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
    fn send_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        if 
            game.current_phase().phase() == PhaseType::Night &&
            !actor_ref.ability_deactivated_from_death(game) &&
            self.jailed_target_ref.is_some()
        {
            out.insert(actor_ref, ChatGroup::Jail);
        }
        if let Some(target) = self.jailed_target_ref && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Jail);
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
            out.insert(actor_ref, ChatGroup::Jail);
        }
        if let Some(target) = self.jailed_target_ref && game.current_phase().phase() == PhaseType::Night {
            out.insert(target, ChatGroup::Jail);
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
                
                actor_ref.edit_role_ability_helper(game, self);

                Detained::add_detain(game, *target);
                actor_ref.add_private_chat_message(game, 
                    ChatMessageVariant::JailedTarget{ player_index: *target }
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