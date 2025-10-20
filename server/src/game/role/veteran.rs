use serde::Serialize;
use crate::game::prelude::*;

#[derive(Debug, Clone)]
pub struct Veteran { 
    alerts_remaining: u8, 
    alerting_tonight: bool 
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    alerts_remaining: u8
}

impl Default for Veteran {
    fn default() -> Self {
        Veteran {
            alerts_remaining: 3,
            alerting_tonight: false
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Veteran {
    type ClientAbilityState = ClientRoleState;
    fn new_state(game: &mut Game) -> Self {
        Self{
            alerts_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::TopPriority => {
                let can_alert = self.alerts_remaining > 0 && game.day_number() > 1;
                let chose_to_alert = matches!(ControllerID::role(actor_ref, Role::Veteran, 0).get_boolean_selection(game), Some(BooleanSelection(true)));
                if can_alert && chose_to_alert{
                    actor_ref.edit_role_ability_helper(game, Veteran { 
                        alerts_remaining: self.alerts_remaining.saturating_sub(1), 
                        alerting_tonight: true 
                    });
                }
            }
            OnMidnightPriority::Heal=>{
                if !self.alerting_tonight {return}
                actor_ref.increase_defense_to(game, midnight_variables, DefensePower::Protected);
            }
            OnMidnightPriority::Kill => {
                if !self.alerting_tonight {return}

                actor_ref.rampage(
                    game,
                    midnight_variables,
                    actor_ref,
                    GraveKiller::Role(Role::Veteran),
                    AttackPower::ArmorPiercing,
                    false,
                    |_|true
                );
            }
            _=>{}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Veteran, 0))
            .available_selection(AvailableBooleanSelection)
            .night_typical(actor_ref)
            .add_grayed_out_condition(self.alerts_remaining == 0 || game.day_number() <= 1)
            .build_map()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.edit_role_ability_helper(
            game,
            Veteran { alerts_remaining: self.alerts_remaining, alerting_tonight: false });   
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut OnMidnightFold, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
impl GetClientAbilityState<ClientRoleState> for Veteran {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            alerts_remaining: self.alerts_remaining
        }
    }
}