use serde::Serialize;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::role_list_generation::criteria::GenerationCriterion;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lovebird {
    pub beloved: Option<PlayerReference>,
    kisses_given: u8,
}

impl Default for Lovebird {
    fn default() -> Self {
        Self { 
            beloved: None,
            kisses_given: 0,
        }
    }
}

impl Lovebird {
    pub fn new_state(_game: &Game) -> Self {
        Self::default()
    }

    pub fn role_list_generation_criteria() -> Vec<GenerationCriterion> {
        vec![
            // No special generation criteria for now
        ]
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Lovebird {
    type ClientRoleState = Self;
    
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Roleblock => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                
                if let Some(visit) = actor_visits.first() {
                    let target = visit.target;
                    
                    // If kissing their beloved, create a special romantic moment
                    if Some(target) == self.beloved {
                        // TODO: Both lovers get protection when they kiss each other
                        
                        // Increase happiness/love meter
                        actor_ref.set_role_state(game, Lovebird {
                            beloved: self.beloved,
                            kisses_given: self.kisses_given.saturating_add(1),
                        });
                    } else {
                        // Kissing someone else (friendly kiss)
                        actor_ref.set_role_state(game, Lovebird {
                            beloved: self.beloved,
                            kisses_given: self.kisses_given.saturating_add(1),
                        });
                    }
                }
            }
            OnMidnightPriority::Kill => {
                // Check if beloved died - if so, die of heartbreak
                if let Some(beloved_ref) = self.beloved {
                    if beloved_ref.alive(game) == false {
                        actor_ref.die(game);
                    }
                }
            }
            _ => {}
        }
    }
    
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Lovebird, 0))
            .single_player_selection_typical(actor_ref, true, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Lovebird, 0),
            false
        )
    }
}

impl Lovebird {
    pub fn set_beloved(&mut self, beloved: PlayerReference) {
        self.beloved = Some(beloved);
    }
    
    pub fn has_beloved(&self) -> bool {
        self.beloved.is_some()
    }
    
    pub fn get_beloved(&self) -> Option<PlayerReference> {
        self.beloved
    }
}
