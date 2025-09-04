use serde::Serialize;
use crate::game::ability::AbilityState;
use crate::game::attack_power::AttackPower;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{game_conclusion::GameConclusion};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::controllers::{ControllerParametersMap, ControllerID};
use crate::game::role::Role;
use crate::game::components::night_visits::{Visits, NightVisitsIterator};
use crate::game::visit::VisitTag;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VigilanteGun {
    state: VigilanteGunState
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum VigilanteGunState {
    NotLoaded,
    Loaded{bullets: u8},
    WillSuicide,
    Suicided,
}

impl Default for VigilanteGun {
    fn default() -> Self {
        Self { state: VigilanteGunState::NotLoaded }
    }
}

impl VigilanteGun {
    pub fn new_state(_game: &Game) -> Self {
        // Vigilante starts as NotLoaded and loads after night 1
        Self {
            state: VigilanteGunState::NotLoaded
        }
    }
    
    pub fn can_shoot(&self) -> bool {
        if let VigilanteGunState::Loaded { bullets } = &self.state {
            *bullets >= 1
        } else {
            false
        }
    }
}

impl AbilityState for VigilanteGun {
    fn on_midnight(&mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::TopPriority => {
                if VigilanteGunState::WillSuicide == self.state {
                    actor_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Suicide, AttackPower::ProtectionPiercing, false);
                    self.state = VigilanteGunState::Suicided;
                }
            },
            OnMidnightPriority::Kill => {
                match self.state {
                    VigilanteGunState::Loaded { bullets } if bullets > 0 => {
                        // Get visits for this specific ability
                        let actor_visits: Vec<Visit> = Visits::iter(midnight_variables)
                            .with_visitor(actor_ref)
                            .filter(|visit| matches!(visit.tag, VisitTag::Ability { ability: crate::game::ability::AbilityID::VigilanteGun, .. }))
                            .copied()
                            .collect();
                            
                        if let Some(visit) = actor_visits.first(){
                            let target_ref = visit.target;

                            let killed = target_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(Role::Vigilante), AttackPower::Basic, false);
                            self.state = VigilanteGunState::Loaded { bullets: bullets.saturating_sub(1) };

                            if killed && target_ref.win_condition(game).is_loyalist_for(GameConclusion::Town) {
                                self.state = VigilanteGunState::WillSuicide;
                            }                            
                        }
                    }       

                    VigilanteGunState::NotLoaded => {
                        self.state = VigilanteGunState::Loaded { bullets: crate::game::role::common_role::standard_charges(game) };
                    }

                    _ => {},
                }
            },
            _ => {}
        }
    }
    
    fn controller_parameters_map(&self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        let can_shoot = if let VigilanteGunState::Loaded { bullets } = &self.state {
            *bullets >= 1
        } else {
            false
        };
        
        ControllerParametersMap::builder(game)
            .id(ControllerID::ability(actor_ref, crate::game::ability::AbilityID::VigilanteGun, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(!can_shoot)
            .build_map()
    }
    
    fn convert_selection_to_visits(&self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
            game,
            actor_ref,
            ControllerID::ability(actor_ref, crate::game::ability::AbilityID::VigilanteGun, 0),
            true,
            crate::game::visit::VisitTag::Ability { ability: crate::game::ability::AbilityID::VigilanteGun, id: 0 }
        )
    }
    
    fn on_player_roleblocked(&mut self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, _invisible: bool) {
        if player != actor_ref {return}

        Visits::retain(midnight_variables, |v|
            !matches!(v.tag, VisitTag::Ability{..}) || v.visitor != actor_ref
        );
    }
    
    fn on_visit_wardblocked(&mut self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
        if actor_ref != visit.visitor {return};

        Visits::retain(midnight_variables, |v|
            !matches!(v.tag, VisitTag::Ability{..}) || v.visitor != actor_ref
        );
    }
}
