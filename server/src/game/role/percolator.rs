use serde::Serialize;

use crate::game::ability_input::{AvailableBooleanSelection, BooleanSelection, ControllerID};
use crate::game::components::confused::Confused;
use crate::game::components::win_condition::WinCondition;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::role::{common_role, GetClientRoleState};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;
use crate::vec_set::VecSet;
use super::{ControllerParametersMap, Role, RoleStateImpl};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Default)]
pub struct Percolator {
    sieve: VecSet<PlayerReference>,
}

#[derive(Clone, Debug, Serialize, Default)]
struct PercolatorClientRoleState;

impl RoleStateImpl for Percolator {
    type ClientRoleState = PercolatorClientRoleState;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let mut new_sieve = self.sieve.clone();

        if let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Percolator, 0).get_boolean_selection(game) {
            new_sieve = PlayerReference::all_players(game).collect();
            actor_ref.set_role_state(game, Percolator { sieve: new_sieve.clone() });
            actor_ref.push_night_message(midnight_variables, 
                ChatMessageVariant::PercolatorResult { sieve: new_sieve }
            );
            return;
        }

        // This is constant throughout a game, but it's probably not expensive enough
        // to warrant caching it.
        // Currently, evils are 40% more likely to stay in the sieve.
        // To change this, change the constant in the `friend_filter_probability` calculation.
        let (enemy_filter_probability, friend_filter_probability, nightly_narrowing_probability) = {
            // We want the sieve to have narrowed to 1 player over 1 + ceil(PLAYERS/5) nights (for balance).
            // We also want enemies to be more likely to be filtered out than friends.
            // That's why this math is a bit complicated.

            let max_narrowing_probability = 1.0 / game.num_players() as f64;
            let desired_max_narrowing_time_nights = 1 + common_role::standard_charges(game);
            let nightly_narrowing_probability = max_narrowing_probability.powf(1.0 / desired_max_narrowing_time_nights as f64);

            let friend_filter_probability = nightly_narrowing_probability * 0.80;
            let enemy_filter_probability = (
                2.0 * nightly_narrowing_probability.powi(desired_max_narrowing_time_nights as i32) 
                - friend_filter_probability.powi(desired_max_narrowing_time_nights as i32)
            ).powf(1.0 / desired_max_narrowing_time_nights as f64);

            (enemy_filter_probability, friend_filter_probability, nightly_narrowing_probability)
        };

        if actor_ref.night_blocked(midnight_variables) {
            // Don't narrow the sieve
        } else if Confused::is_confused(game, actor_ref) {
            new_sieve = self.sieve.iter()
                .filter(|_| rand::random_range(0.0..=1.0) < nightly_narrowing_probability)
                .copied()
                .collect();
        } else {
            new_sieve = self.sieve.iter()
                .filter(|p| {
                    if WinCondition::are_friends(p.win_condition(game), actor_ref.win_condition(game)) {
                        rand::random_range(0.0..=1.0) < friend_filter_probability
                    } else {
                        rand::random_range(0.0..=1.0) < enemy_filter_probability
                    }
                })
                .copied()
                .collect()
        }

        actor_ref.set_role_state(game, Percolator { sieve: new_sieve.clone() });
        actor_ref.push_night_message(midnight_variables, 
            ChatMessageVariant::PercolatorResult { sieve: new_sieve }
        );
    }
    fn on_game_start(self, game: &mut Game, actor_ref: PlayerReference) {
        actor_ref.set_role_state(game, Percolator {
            sieve: PlayerReference::all_players(game).collect(),
        });
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Percolator, 0))
            .available_selection(AvailableBooleanSelection)
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Percolator, 0),
            false
        )
    }
}

impl GetClientRoleState<PercolatorClientRoleState> for Percolator {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> PercolatorClientRoleState {
        PercolatorClientRoleState
    }
}

impl Percolator {
    pub fn player_is_suspicious(game: &Game, midnight_variables: &MidnightVariables, player_ref: PlayerReference) -> bool {
        if player_ref.has_suspicious_aura(game, midnight_variables){
            true
        }else if player_ref.has_innocent_aura(game){
            false
        }else{
            !player_ref.win_condition(game).friends_with_conclusion(GameConclusion::Town)
        }
    }
}