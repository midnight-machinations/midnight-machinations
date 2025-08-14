
use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, game_conclusion::GameConclusion};
use crate::game::grave::GraveKiller;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleState, RoleStateImpl};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vigilante {
    state: VigilanteState
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum VigilanteState{
    NotLoaded,
    Loaded{bullets: u8},
    WillSuicide,
    Suicided,
}

impl Default for Vigilante {
    fn default() -> Self {
        Self { state: VigilanteState::NotLoaded }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Vigilante {
    type ClientRoleState = Vigilante;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority{
            OnMidnightPriority::TopPriority => {
                if VigilanteState::WillSuicide == self.state {
                    actor_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Suicide, AttackPower::ProtectionPiercing, false);
                    self.state = VigilanteState::Suicided;
                }
            },
            OnMidnightPriority::Kill => {
            
                match self.state {
                    VigilanteState::Loaded { bullets } if bullets > 0 => {
                        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                        if let Some(visit) = actor_visits.first(){

                            let target_ref = visit.target;

                            let killed = target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Vigilante), AttackPower::Basic, false);
                            self.state = VigilanteState::Loaded { bullets: bullets.saturating_sub(1) };

                            if killed && target_ref.win_condition(game).is_loyalist_for(GameConclusion::Town) {
                                self.state = VigilanteState::WillSuicide;
                            }                            
                        }
                    }       

                    VigilanteState::NotLoaded => {
                        self.state = VigilanteState::Loaded { bullets: crate::game::role::common_role::standard_charges(game) };
                    }

                    _ => {},
                    
                }
            },
            _ => {}
        }
    actor_ref.set_role_state(game, RoleState::Vigilante(self));
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        let can_shoot = if let VigilanteState::Loaded { bullets } = &self.state {
            *bullets >=1
        } else {
            false
        };
        
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Vigilante, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(!can_shoot)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Vigilante, 0),
            true
        )
    }
}