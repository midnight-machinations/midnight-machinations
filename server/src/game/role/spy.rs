use rand::seq::SliceRandom;
use serde::Serialize;

use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::{attack_power::DefensePower, event::on_midnight::OnMidnightPriority};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Spy;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Spy {
    type ClientRoleState = Spy;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let Some(bugged) = Visits::default_target(game, midnight_variables, actor_ref) else {return};

        let mut roles: Vec<Role> = Visits::into_iter(midnight_variables)
            .with_appeared(midnight_variables)
            .with_insider_visitor(game, InsiderGroupID::Mafia)
            .with_target(bugged)
            .map_visitor()
            .map(|p|p.role(game))
            .collect();
        roles.shuffle(&mut rand::rng());

        let mut syndicate_visited_players: Vec<PlayerReference> = Visits::into_iter(midnight_variables)
            .with_appeared(midnight_variables)
            .with_insider_visitor(game, InsiderGroupID::Mafia)
            .map_target()
            .collect();
        syndicate_visited_players.shuffle(&mut rand::rng());
        
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyMafiaVisit { players: syndicate_visited_players });
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyBug { roles } );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Spy, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Spy, 0),
            false
        )
    }
}