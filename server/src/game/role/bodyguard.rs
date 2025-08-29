
use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::attack_power::DefensePower;
use crate::game::components::transport::Transport;
use crate::game::components::transport::TransportPriority;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;

use crate::vec_map::vec_map;

use super::{
    ControllerID, ControllerParametersMap,
    GetClientRoleState, Role,
    RoleStateImpl
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bodyguard {
    self_shields_remaining: u8,
    redirected_player_refs: Vec<PlayerReference>
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    self_shields_remaining: u8
}

impl Default for Bodyguard {
    fn default() -> Self {
        Self { 
            self_shields_remaining: 1, 
            redirected_player_refs: Vec::new()
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Bodyguard {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return};
        
        match priority {
            OnMidnightPriority::Bodyguard => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(target_ref) = actor_visits.get(0).map(|v| v.target) else {return};
                
                if actor_ref == target_ref {return}
                
                let redirected_player_refs = Transport::transport(
                    midnight_variables, TransportPriority::Bodyguard,
                    &vec_map![(target_ref, actor_ref)], |v| v.attack, false, 
                ).iter().filter(|v|!v.indirect).map(|v| v.visitor).collect();

                actor_ref.set_role_state(game, Bodyguard {
                    redirected_player_refs,
                    ..self
                });
                
            },
            OnMidnightPriority::Heal => {
                let actors_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actors_visits.first() else {return};
                let target_ref = visit.target;
    
                if actor_ref == target_ref {
                    let self_shields_remaining = self.self_shields_remaining.saturating_sub(1);
                    actor_ref.set_role_state(game, Bodyguard{
                        self_shields_remaining, 
                        ..self
                    });
                    
                    actor_ref.guard_player(game, midnight_variables, target_ref);
                }
            },
            OnMidnightPriority::Kill => {
                for redirected_player_ref in self.redirected_player_refs {
                    redirected_player_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Bodyguard), AttackPower::ArmorPiercing, false);
                }
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Bodyguard, 0))
            .single_player_selection_typical(actor_ref, self.self_shields_remaining > 0, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Bodyguard, 0),
            false
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.set_role_state(game, Bodyguard {
            self_shields_remaining: self.self_shields_remaining,
            redirected_player_refs: Vec::new(),
        });
    }
}
impl GetClientRoleState<ClientRoleState> for Bodyguard {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            self_shields_remaining: self.self_shields_remaining
        }
    }
}