
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::attack_power::DefensePower;
use crate::game::components::night_visits::NightVisits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::game_conclusion::GameConclusion;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap,
    Role, RoleStateImpl
};



#[derive(Clone, Debug, Default, Serialize)]
pub struct Cop;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Cop {
    type ClientRoleState = Self;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}

        match priority {
            OnMidnightPriority::Heal => {
                
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actor_visits.first() else {return};
                let target_ref = visit.target;

                actor_ref.guard_player(game, midnight_variables, target_ref);
            }
            OnMidnightPriority::Kill => {
                
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(ambush_visit) = actor_visits.first() else {return};
                let target_ref = ambush_visit.target;

                let player_to_attacks_visit = 
                if let Some(priority_visitor) = NightVisits::all_visits(midnight_variables).into_iter()
                    .filter(|visit|
                        ambush_visit != *visit &&
                        visit.target == target_ref &&
                        visit.visitor.alive(game) &&
                        !visit.visitor.win_condition(game).is_loyalist_for(GameConclusion::Town)
                    ).collect::<Vec<&Visit>>()
                    .choose(&mut rand::rng())
                    .copied()
                {
                    Some(priority_visitor.visitor)
                } else {
                    NightVisits::all_visits(midnight_variables).into_iter()
                        .filter(|visit|
                            ambush_visit != *visit &&
                            visit.target == target_ref &&
                            visit.visitor.alive(game)
                        ).collect::<Vec<&Visit>>()
                        .choose(&mut rand::rng())
                        .copied()
                        .map(|v|v.visitor)
                };

                if let Some(player_to_attack) = player_to_attacks_visit{
                    player_to_attack.try_night_kill_single_attacker(
                        actor_ref,
                        game,
                        midnight_variables,
                        GraveKiller::Role(Role::Cop),
                        AttackPower::Basic,
                        false
                    );
                }
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Cop, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Cop, 0),
            false
        )
    }
}