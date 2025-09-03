use serde::Serialize;

use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};
use crate::game::chat::ChatMessageVariant;
use crate::game::phase::PhaseType;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl, GetClientRoleState};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Business {
    pub bribes_remaining: u8,
    pub bribed_players: Vec<PlayerReference>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    pub bribes_remaining: u8,
}

impl Default for Business {
    fn default() -> Self {
        Business {
            bribes_remaining: 3,
            bribed_players: Vec::new(),
        }
    }
}

impl GetClientRoleState<ClientRoleState> for Business {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            bribes_remaining: self.bribes_remaining,
        }
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Business {
    type ClientRoleState = ClientRoleState;
    
    fn new_state(game: &Game) -> Self {
        Self {
            bribes_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }

    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception { return; }
        if self.bribes_remaining == 0 { return; }

        if let Some(visit) = Visits::default_visit(game, midnight_variables, actor_ref) {
            let target_ref = visit.target;

            // Bribe the target - they cannot vote the next day
            let mut new_state = self.clone();
            new_state.bribes_remaining = new_state.bribes_remaining.saturating_sub(1);
            new_state.bribed_players.push(target_ref);
            
            actor_ref.set_role_state(game, new_state);

            // Send a message to the businessman about the successful bribe
            actor_ref.push_night_message(midnight_variables, ChatMessageVariant::PlayerQuit { 
                player_index: target_ref.index(),
                game_over: false
            });
        }
    }

    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Business, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(self.bribes_remaining == 0)
            .build_map()
    }

    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Business, 0),
            false
        )
    }

    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }

    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType) {
        if phase == PhaseType::Discussion {
            // Remove bribed players from the list at the start of each new day
            if !self.bribed_players.is_empty() {
                let new_state = Business {
                    bribes_remaining: self.bribes_remaining,
                    bribed_players: Vec::new(),
                };
                actor_ref.set_role_state(game, new_state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::components::insider_group::InsiderGroupID;

    #[test]
    fn business_has_correct_defense() {
        assert_eq!(DEFENSE, DefensePower::None);
    }

    #[test]
    fn business_has_correct_max_count() {
        assert_eq!(MAXIMUM_COUNT, Some(1));
    }

    #[test]
    fn business_default_state() {
        let business = Business::default();
        assert_eq!(business.bribes_remaining, 3);
        assert_eq!(business.bribed_players.len(), 0);
    }

    #[test]
    fn business_is_mafia() {
        let business = Business::default();
        let revealed_groups = business.default_revealed_groups();
        assert!(revealed_groups.contains(&InsiderGroupID::Mafia));
    }
}
