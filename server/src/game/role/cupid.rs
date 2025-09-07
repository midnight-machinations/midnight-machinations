use serde::Serialize;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::role_list_generation::criteria::GenerationCriterion;
use super::{ControllerID, ControllerParametersMap, Role, RoleState, RoleStateImpl};
use super::lovebird::Lovebird;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cupid {
    arrows_remaining: u8,
    matched_players: Vec<(PlayerReference, PlayerReference)>,
    used_match_ability: bool,
}

impl Default for Cupid {
    fn default() -> Self {
        Self { 
            arrows_remaining: 2,
            matched_players: Vec::new(),
            used_match_ability: false,
        }
    }
}

impl Cupid {
    pub fn new_state(_game: &Game) -> Self {
        Self::default()
    }

    pub fn role_list_generation_criteria() -> Vec<GenerationCriterion> {
        vec![
            // No special generation criteria for now
        ]
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Cupid {
    type ClientRoleState = Self;
    
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Roleblock => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                
                                // Match ability - two targets
                if actor_visits.len() == 2 && !self.used_match_ability {
                    let player1 = actor_visits[0].target;
                    let player2 = actor_visits[1].target;
                    
                    // Create match pair
                    let mut updated_pairs = self.matched_players.clone();
                    updated_pairs.push((player1, player2));
                    
                    actor_ref.set_role_state(game, Cupid {
                        arrows_remaining: self.arrows_remaining,
                        matched_players: updated_pairs,
                        used_match_ability: true,
                    });
                }
                
                // Kiss ability - single target
                else if actor_visits.len() == 1 {
                    let _target = actor_visits[0].target;
                    
                    // TODO: Add protection functionality
                    
                    actor_ref.set_role_state(game, Cupid {
                        arrows_remaining: self.arrows_remaining.saturating_sub(1),
                        matched_players: self.matched_players,
                        used_match_ability: self.used_match_ability,
                    });
                }
            }
            _ => {}
        }
    }
    
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Cupid, 0))
            .single_player_selection_typical(actor_ref, true, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        if !self.used_match_ability && self.arrows_remaining >= 2 {
            // Double selection for matching
            crate::game::role::common_role::convert_controller_selection_to_visits(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Cupid, 0),
                false
            )
        } else {
            // Single selection for kiss
            crate::game::role::common_role::convert_controller_selection_to_visits(
                game,
                actor_ref,
                ControllerID::role(actor_ref, Role::Cupid, 1),
                false
            )
        }
    }
}

impl Cupid {
    fn create_love_bond(&self, game: &mut Game, _cupid_ref: PlayerReference, player1: PlayerReference, player2: PlayerReference) {
        // Convert both players to Lovebirds with each other as beloveds
        let mut lovebird1 = Lovebird::new_state(game);
        lovebird1.set_beloved(player2);
        
        let mut lovebird2 = Lovebird::new_state(game);
        lovebird2.set_beloved(player1);
        
        player1.set_role_and_win_condition_and_revealed_group(game, RoleState::Lovebird(lovebird1));
        player2.set_role_and_win_condition_and_revealed_group(game, RoleState::Lovebird(lovebird2));
    }
}
