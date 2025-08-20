use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::controllers::ControllerParametersMap;
use crate::game::components::fragile_vest::FragileVests;
use crate::game::components::player_component::PlayerComponent;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;
use crate::vec_set;
use super::{common_role, ControllerID, GetClientRoleState, Role, RoleStateImpl};

#[derive(Clone, Debug)]
pub struct Armorsmith {
    open_shops_remaining: u8,
    night_open_shop: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientRoleState {
    open_shops_remaining: u8
}

impl Default for Armorsmith {
    fn default() -> Self {
        Self { 
            open_shops_remaining: 3,
            night_open_shop: false,
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Armorsmith {
    type ClientRoleState = ClientRoleState;
    fn on_midnight(mut self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        match priority {
            OnMidnightPriority::Heal => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                if let Some(visit) = actor_visits.first() && self.open_shops_remaining != 0 {
                    self.night_open_shop = true;
                    self.open_shops_remaining = self.open_shops_remaining.saturating_sub(1);

                    actor_ref.guard_player(game, midnight_variables, actor_ref);


                    let visitors = actor_ref.all_night_visitors_cloned(midnight_variables);

                    for visitor in visitors.iter(){
                        actor_ref.guard_player(game, midnight_variables, *visitor);
                    }

                    if visitors.contains(&visit.target){
                        PlayerComponent::<FragileVests>::add_defense_item_midnight(game, midnight_variables, visit.target, DefensePower::Protected, vec_set![actor_ref]);
                    }else if let Some(random_visitor) = visitors.choose(&mut rand::rng()) {
                        PlayerComponent::<FragileVests>::add_defense_item_midnight(game, midnight_variables, *random_visitor, DefensePower::Protected, vec_set![actor_ref]);
                    }
                };

                actor_ref.set_role_state(game, self);
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Armorsmith, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(self.open_shops_remaining == 0)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<crate::game::visit::Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Armorsmith, 0),
            false
        )
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        actor_ref.set_role_state(game, 
            Armorsmith{
                night_open_shop: false,
                ..self
            });
    }
    fn new_state(game: &Game) -> Self {
        Self{
            open_shops_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
}
impl GetClientRoleState<ClientRoleState> for Armorsmith {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            open_shops_remaining: self.open_shops_remaining
        }
    }
}