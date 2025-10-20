use serde::Serialize;
use crate::game::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Escort;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Escort {
    type ClientAbilityState = Escort;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Roleblock {return;}
        if let Some(visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Escort) {
            let target_ref = visit.target;

            target_ref.roleblock(game, midnight_variables, true);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Escort, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Escort, 0),
            false
        )
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut OnMidnightFold, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
