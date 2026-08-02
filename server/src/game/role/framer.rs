use serde::Serialize;
use crate::game::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Framer;

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Framer {
    type ClientAbilityState = Framer;
    fn on_midnight(self, _game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Deception {return}
        
        let Some(framed) = Visits::default_target(midnight_variables, actor_ref, Role::Framer) else {return};
        
        framed.set_night_framed(midnight_variables, true);
        framed.set_night_appeared_visits(midnight_variables, true);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Framer, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Framer, 0),
            false,
        )
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
}