use rand::prelude::SliceRandom;
use serde::Serialize;
use crate::game::prelude::*;

#[derive(Clone, Serialize, Debug, Default)]
pub struct Spotter;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Spotter {
    type ClientAbilityState = Spotter;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}
        let Some(visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Spotter) else {return};
        
        let spotable = ControllerID::role(actor_ref, Role::Spotter, 1)
            .get_player_list_selection(game)
            .map(|s| s.0.clone())
            .unwrap_or_default();

        let mut players: Vec<PlayerReference> = visit.
            target
            .lookout_seen_players(midnight_variables, visit)
            .filter(|p| spotable.contains(p))
            .collect();

        players.shuffle(&mut game.rng);
        
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::LookoutResult { players });
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Spotter, 1))
                .player_list_selection_typical(actor_ref, false, true, false, false, true, Some(3))
                .night_typical(actor_ref)
                .build_map(),
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Spotter, 0))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .build_map()
        ])
    }
    fn create_visits_initialize_night(self, game: &Game, _id: &AbilityID, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Spotter, 0),
            false
        )
    }
}