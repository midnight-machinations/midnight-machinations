use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*};

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
            OnMidnightPriority::Heal=>{
                let can_alert = self.alerts_remaining > 0 && game.day_number() > 1;
                let chose_to_alert = ControllerID::role(actor_ref, Role::Veteran, 0)
                    .get_player_list_selection(game)
                    .and_then(|players|players.0.first())
                    .is_some();
                
                
                if can_alert && chose_to_alert{
                    actor_ref.edit_role_ability_helper(game, Veteran { 
                        alerts_remaining: self.alerts_remaining.saturating_sub(1), 
                        alerting_tonight: true 
                    });
                }

                if !self.alerting_tonight {return}
                actor_ref.increase_defense_to(game, midnight_variables, DefensePower::Protected);
            }
            OnMidnightPriority::Kill => {
                if !self.alerting_tonight {return}

                let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Veteran) else {return};

                NightAttack::new()
                    .attackers([actor_ref])
                    .grave_killer(Role::Veteran)
                    .power(AttackPower::ArmorPiercing)
                    .rampage(
                        game,
                        midnight_variables,
                        actor_ref,
                        |v| v.visitor != target
                    );
            }
            _=>{}
        }
    }
    fn create_visits_initialize_night(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Veteran, 0),
            false
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Veteran, 0))
            .single_player_selection_typical(actor_ref, true, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(self.alerts_remaining == 0 || game.day_number() <= 1)
            .build_map()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.edit_role_ability_helper(
            game,
            Veteran { alerts_remaining: self.alerts_remaining, alerting_tonight: false });   
    }
    fn on_player_roleblocked(self, _game: &mut Game, _id: &AbilityID, _event: &OnPlayerRoleblocked, _fold: &mut OnMidnightFold, _priority: ()) {}
}
impl GetClientAbilityState<ClientRoleState> for Veteran {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            alerts_remaining: self.alerts_remaining
        }
    }
}