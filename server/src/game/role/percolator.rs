use serde::Serialize;

use crate::game::controllers::{AvailableBooleanSelection, BooleanSelection, ControllerID};
use crate::game::components::confused::Confused;
use crate::game::components::win_condition::WinCondition;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::role::{common_role, GetClientRoleState};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;
use crate::vec_set::VecSet;
use super::{ControllerParametersMap, Role, RoleStateImpl};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Default)]
pub struct Alien {
    sieve: VecSet<PlayerReference>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct AlienClientRoleState;

impl RoleStateImpl for Alien {
    type ClientRoleState = AlienClientRoleState;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let new_sieve: VecSet<PlayerReference>;

        if let Some(BooleanSelection(true)) = ControllerID::role(actor_ref, Role::Alien, 0).get_boolean_selection(game) {
            if actor_ref.night_blocked(midnight_variables) {
                return;
            }

            new_sieve = PlayerReference::all_players(game).collect();
            actor_ref.set_role_state(game, Alien { sieve: new_sieve.clone() });
            actor_ref.push_night_message(midnight_variables, 
                ChatMessageVariant::AlienResult { sieve: new_sieve }
            );
            return;
        }

        let (enemy_filter_probability, friend_filter_probability, nightly_narrowing_probability) = Self::get_probabilities(game);

        if actor_ref.night_blocked(midnight_variables) {
            // Don't narrow the sieve
            return;
        } else if Confused::is_confused(game, actor_ref) {
            new_sieve = self.try_filter_sieve(|_player| {
                rand::random_range(0.0..=1.0) < nightly_narrowing_probability
            });
        } else {
            new_sieve = self.try_filter_sieve(|player| {
                if WinCondition::are_friends(player.win_condition(game), actor_ref.win_condition(game)) {
                    rand::random_range(0.0..=1.0) < friend_filter_probability
                } else {
                    rand::random_range(0.0..=1.0) < enemy_filter_probability
                }
            });
        }

        actor_ref.set_role_state(game, Alien { sieve: new_sieve.clone() });
        actor_ref.push_night_message(midnight_variables, 
            ChatMessageVariant::AlienResult { sieve: new_sieve }
        );
    }
    #[expect(clippy::cast_possible_truncation, reason = "We want to send u8s, not f64s")]
    #[expect(clippy::cast_sign_loss, reason = "We want to send u8s, not f64s")]
    fn on_game_start(self, game: &mut Game, actor_ref: PlayerReference) {
        let new_sieve = PlayerReference::all_players(game).collect::<VecSet<_>>();
        actor_ref.set_role_state(game, Alien { sieve: new_sieve.clone() });
        actor_ref.add_private_chat_message(game, ChatMessageVariant::AlienResult { sieve: new_sieve });

        let (enemy_filter_probability, friend_filter_probability, _) = Self::get_probabilities(game);

        let enemy_filter_probability = (enemy_filter_probability * 255.0) as u8;
        let friend_filter_probability = (friend_filter_probability * 255.0) as u8;

        actor_ref.add_private_chat_message(game, ChatMessageVariant::AlienProbabilities { enemy_filter_probability, friend_filter_probability });
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Alien, 0))
            .available_selection(AvailableBooleanSelection)
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Alien, 0),
            false
        )
    }
}

impl GetClientRoleState<AlienClientRoleState> for Alien {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> AlienClientRoleState {
        AlienClientRoleState
    }
}

impl Alien {
    fn try_filter_sieve(self, filter_algorithm: impl Fn(PlayerReference) -> bool) -> VecSet<PlayerReference> {
        let mut tries = 0u8;

        if self.sieve.is_empty() {
            return VecSet::new();
        }

        loop {
            let filtered = self.sieve.iter()
                .filter(|p| filter_algorithm(**p))
                .copied()
                .collect::<VecSet<_>>();

            if filtered.count() < self.sieve.count() {
                return filtered;
            }

            tries = tries.saturating_add(1);

            if tries >= 100 {
                // If we can't filter out any players after 100 tries, just remove a random player.
                // This will not mess with probabilities that much. Probably.
                let mut new_sieve = self.sieve.iter().copied().collect::<Vec<_>>();

                let random_index = rand::random_range(0..self.sieve.count());
                new_sieve.remove(random_index);
                
                return new_sieve.into_iter().collect();
            }
        }
    }

    // This is constant throughout a game, but it's probably not expensive enough
    // to warrant caching it.
    // Currently, evils are 40% more likely to stay in the sieve.
    // To change this, change the constant in the `friend_filter_probability` calculation.
    fn get_probabilities(game: &Game) -> (f64, f64, f64) {
        // We want the sieve to have narrowed to 1 player over 1 + ceil(PLAYERS/5) nights (for balance).
        // We also want enemies to be more likely to be filtered out than friends.
        // That's why this math is a bit complicated.

        let max_narrowing_probability = 1.0 / game.num_players() as f64;
        let desired_max_narrowing_time_nights = common_role::standard_charges(game).saturating_add(1);
        let nightly_narrowing_probability = max_narrowing_probability.powf(1.0 / desired_max_narrowing_time_nights as f64);

        let friend_filter_probability = nightly_narrowing_probability * 0.80;
        let enemy_filter_probability = (
            2.0 * nightly_narrowing_probability.powi(desired_max_narrowing_time_nights as i32) 
            - friend_filter_probability.powi(desired_max_narrowing_time_nights as i32)
        ).powf(1.0 / desired_max_narrowing_time_nights as f64);

        // Yeah this return type is mid but just make sure to update callers if you change this order.
        (enemy_filter_probability, friend_filter_probability, nightly_narrowing_probability)
    }
}