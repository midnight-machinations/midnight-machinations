use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*};

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

impl RoleStateTrait for Vigilante {
    type ClientAbilityState = Vigilante;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        match priority{
            OnMidnightPriority::TopPriority => {
                if VigilanteState::WillSuicide == self.state {
                    NightAttack::new()
                        .attackers([actor_ref])
                        .grave_killer(GraveKiller::Suicide)
                        .power(AttackPower::ProtectionPiercing)
                        .attack(game, midnight_variables, actor_ref);
                    self.state = VigilanteState::Suicided;
                }
            },
            OnMidnightPriority::Kill => {
                match self.state {
                    VigilanteState::Loaded { bullets } if bullets > 0 => {
                        if let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Vigilante) {
                            let killed = NightAttack::new()
                                .attackers([actor_ref])
                                .grave_killer(Role::Vigilante)
                                .attack(game, midnight_variables, target_ref);
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
        
        actor_ref.edit_role_ability_helper(game, self);
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