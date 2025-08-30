use rand::prelude::SliceRandom;
use serde::Serialize;

use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Serialize, Debug, Default)]
pub struct Lookout;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Lookout {
    type ClientRoleState = Lookout;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}
        if let Some(visit) = Visits::default_visit(game, midnight_variables, actor_ref) {
            
            let mut seen_players: Vec<PlayerReference> = visit.target.all_appeared_visitors(game, midnight_variables).into_iter().filter(|p|actor_ref!=*p).collect();
            seen_players.shuffle(&mut rand::rng());

            let message = ChatMessageVariant::LookoutResult { players:
                PlayerReference::ref_vec_to_index(seen_players.as_slice())
            };
            
            actor_ref.push_night_message(midnight_variables, message);
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Lookout, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Lookout, 0),
            false
        )
    }
}