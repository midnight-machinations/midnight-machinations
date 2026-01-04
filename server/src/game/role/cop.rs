use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Cop;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Cop {
    type ClientAbilityState = Self;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}

        match priority {
            OnMidnightPriority::Heal => {
                let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Cop) else {return};

                actor_ref.guard_player(game, midnight_variables, target_ref);
            }
            OnMidnightPriority::Kill => {
                let Some(ambush_visit) = Visits::default_visit(midnight_variables, actor_ref, Role::Cop) else {return};

                if let Some(player_to_attack) = Visits::into_iter(midnight_variables)
                    .without_visit(ambush_visit)
                    .with_target(ambush_visit.target)
                    .with_alive_visitor(game)
                    .without_loyalist_visitor(game, GameConclusion::Town)
                    .with_direct()
                    .map_visitor()
                    .collect::<Box<[PlayerReference]>>()
                    .choose(&mut game.rng)
                    .copied()
                    .or_else(||Visits::into_iter(midnight_variables)
                        .without_visit(ambush_visit)
                        .with_target(ambush_visit.target)
                        .with_alive_visitor(game)
                        .with_direct()
                        .map_visitor()
                        .collect::<Box<[PlayerReference]>>()
                        .choose(&mut game.rng)
                        .copied()
                    )
                {
                    NightAttack::new()
                        .attackers([actor_ref])
                        .grave_killer(Role::Cop)
                        .attack(game, midnight_variables, player_to_attack);
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