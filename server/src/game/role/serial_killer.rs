use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{Role, RoleStateImpl};
use crate::game::controllers::*;

#[derive(Debug, Clone, Serialize, Default)]
pub struct SerialKiller;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateImpl for SerialKiller {
    type ClientRoleState = SerialKiller;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Kill {return}
        if game.day_number() == 1 {return}


        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){

            let target_ref = visit.target;
            
            target_ref.try_night_kill_single_attacker(
                actor_ref,
                game,
                midnight_variables,
                GraveKiller::Role(Role::SerialKiller),
                AttackPower::ArmorPiercing,
                true
            );
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::SerialKiller, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::SerialKiller, 0),
            true
        )
    }
}
