use serde::Serialize;

use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::controllers::AvailableTwoPlayerOptionSelection;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, player::PlayerReference};


use crate::game::visit::Visit;
use crate::game::Game;
use crate::vec_set;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};

#[derive(Clone, Debug, Default, Serialize)]
pub struct Courtesan{
    previous: Vec<PlayerReference>
}

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Courtesan {
    type ClientAbilityState = Courtesan;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Roleblock {return;}
        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let mut previous = Vec::new();
        for visit in actor_visits{
            previous.push(visit.target);
            visit.target.roleblock(game, midnight_variables, true);
        }
        self.previous = previous;
        actor_ref.edit_role_ability_helper(game, self);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        let available_players: vec_set::VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p|
                p.alive(game) &&
                *p != actor_ref &&
                !self.previous.contains(p)
            )
            .collect();

        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Courtesan, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: available_players.clone(),
                available_second_players: available_players,
                can_choose_duplicates: false,
                can_choose_none: true
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Courtesan, 0),
            false
        )
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
