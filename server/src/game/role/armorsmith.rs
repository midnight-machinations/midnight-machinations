use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::components::night_visits::Visits;
use crate::game::controllers::ControllerParametersMap;
use crate::game::components::fragile_vest::FragileVests;
use crate::game::components::player_component::PlayerComponent;
use crate::game::event::on_midnight::{OnMidnightFold, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;
use crate::vec_set;
use super::{common_role, ControllerID, GetClientAbilityState, Role, RoleStateTrait};

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

impl RoleStateTrait for Armorsmith {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Heal {return;}
        let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::Armorsmith) else {return};
        if self.open_shops_remaining == 0 {return}

        self.night_open_shop = true;
        self.open_shops_remaining = self.open_shops_remaining.saturating_sub(1);

        actor_ref.guard_player(game, midnight_variables, actor_ref);

        let visitors: Vec<PlayerReference> = actor_ref.all_direct_night_visitors_cloned(midnight_variables).collect();

        if let Some(player) = if visitors.contains(&target){
            Some(target)
        }else { 
            visitors.choose(&mut game.rng).copied() 
        }{
            PlayerComponent::<FragileVests>::add_defense_item_midnight(
                game,
                midnight_variables,
                player,
                DefensePower::Protected,
                vec_set![actor_ref]
            );
        }

        for visitor in visitors{
            actor_ref.guard_player(game, midnight_variables, visitor);
        }
        

        actor_ref.edit_role_ability_helper(game, self);
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
        actor_ref.edit_role_ability_helper(game, 
            Armorsmith{
                night_open_shop: false,
                ..self
            });
    }
    fn new_state(game: &mut Game) -> Self {
        Self{
            open_shops_remaining: crate::game::role::common_role::standard_charges(game),
            ..Self::default()
        }
    }
}
impl GetClientAbilityState<ClientRoleState> for Armorsmith {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState {
            open_shops_remaining: self.open_shops_remaining
        }
    }
}