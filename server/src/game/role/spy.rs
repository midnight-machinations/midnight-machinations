use rand::seq::SliceRandom;
use serde::Serialize;

use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::role_list::RoleSet;
use crate::game::{attack_power::DefensePower, event::on_midnight::OnMidnightPriority};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::player::{PlayerIndex, PlayerReference};

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

        let my_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(my_visit) = my_visits.first() else {return};

        let mut mafia_visits: Vec<PlayerIndex> = 
        PlayerReference::all_players(game)
            .filter(|p|InsiderGroupID::Mafia.contains_player(game, *p))
            .flat_map(|p|
                p.tracker_seen_visits(game, midnight_variables)
                    .into_iter()
                    .map(|v|v.target.index())
            )
            .collect();
        
        mafia_visits.shuffle(&mut rand::rng());

        let mut roles: Vec<Role> = Visits::into_iter(midnight_variables)
            .filter(|v|v.target == my_visit.target)
            .map(|v|v.visitor)
            .filter_map(|p|
                if RoleSet::Mafia.get_roles().contains(&p.role(game)) {Some(p.role(game))} else {None}
            )
            .collect();
        
        roles.shuffle(&mut rand::rng());
        
        actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyMafiaVisit { players: mafia_visits });
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