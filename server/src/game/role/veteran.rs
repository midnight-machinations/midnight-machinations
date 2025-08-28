use serde::Serialize;

use crate::game::controllers::{AvailableBooleanSelection, ControllerParametersMap};
use crate::game::attack_power::AttackPower;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;

use super::{BooleanSelection, ControllerID, GetClientRoleState, Role, RoleStateImpl};

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

impl RoleStateImpl for Veteran {
    type ClientRoleState = ClientRoleState;
    fn new_state(game: &Game) -> Self {
        Self{
            alerts_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::TopPriority => {
                let can_alert = self.alerts_remaining > 0 && game.day_number() > 1;
                let chose_to_alert = matches!(ControllerID::role(actor_ref, Role::Veteran, 0).get_boolean_selection(game), Some(BooleanSelection(true)));
                if can_alert && chose_to_alert{
                    actor_ref.set_role_state(game, Veteran { 
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
                    |v|true
                );
                for other_player_ref in actor_ref.all_night_visitors_cloned(midnight_variables)
                    .into_iter().filter(|other_player_ref|
                        other_player_ref.alive(game) &&
                        *other_player_ref != actor_ref &&
                        !visit.indirect
                    ).collect::<Vec<PlayerReference>>()
                {
                    other_player_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Veteran), AttackPower::ArmorPiercing, false);
                }
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
        actor_ref.set_role_state(
            game,
            Veteran { alerts_remaining: self.alerts_remaining, alerting_tonight: false });   
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
impl GetClientRoleState<ClientRoleState> for Veteran {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            alerts_remaining: self.alerts_remaining
        }
    }
}