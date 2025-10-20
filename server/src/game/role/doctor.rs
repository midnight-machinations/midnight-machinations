
use serde::Serialize;
use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{OnMidnightFold, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Doctor {
    self_heals_remaining: u8,
}
impl Default for Doctor {
    fn default() -> Self {
        Self { 
            self_heals_remaining: 1
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Doctor {
    type ClientAbilityState = Self;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Heal => {
                let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Doctor) else {return};

                actor_ref.guard_player(game, midnight_variables, target_ref);

                if actor_ref == target_ref{
                    actor_ref.edit_role_ability_helper(game, Doctor{
                        self_heals_remaining: self.self_heals_remaining.saturating_sub(1), 
                    });
                }

            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Doctor, 0))
            .single_player_selection_typical(actor_ref, self.self_heals_remaining > 0, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Doctor, 0),
            false
        )
    }
}