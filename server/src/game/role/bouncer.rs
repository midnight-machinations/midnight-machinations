use serde::Serialize;

use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};


use crate::game::visit::Visit;
use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Bouncer;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Bouncer {
    type ClientRoleState = Bouncer;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if let Some(target) = Visits::default_target(game, midnight_variables, actor_ref) {
            target.ward_night_action(game, midnight_variables, priority);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Bouncer, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Bouncer, 0),
            false
        ).into_iter().map(|mut v|{v.wardblock_immune = true; v}).collect()
    }
    fn on_visit_wardblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _visit: Visit) {}
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
