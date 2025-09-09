
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::components::night_visits::Visits;
use crate::game::components::night_visits::NightVisitsIterator;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::event::on_midnight::OnMidnightPriority;
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap,
    Role, RoleStateTrait
};



#[derive(Clone, Debug, Default, Serialize)]
pub struct Ambusher;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Ambusher {
    type ClientAbilityState = Ambusher;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}

        match priority {
            OnMidnightPriority::Kill => {
                let Some(ambush_visit) = Visits::default_visit(game, midnight_variables, actor_ref) else {return};

                
                if let Some(player_to_attack) = Visits::into_iter(midnight_variables)
                    .without_visit(ambush_visit)
                    .with_target(ambush_visit.target)
                    .with_alive_visitor(game)
                    .with_direct()
                    .with_loyalist_visitor(game, GameConclusion::Town)
                    .map_visitor()
                    .collect::<Box<[PlayerReference]>>()
                    .choose(&mut rand::rng())
                    .copied()
                    .or_else(||Visits::into_iter(midnight_variables)
                        .without_visit(ambush_visit)
                        .with_target(ambush_visit.target)
                        .with_alive_visitor(game)
                        .with_direct()
                        .map_visitor()
                        .collect::<Box<[PlayerReference]>>()
                        .choose(&mut rand::rng())
                        .copied()
                    )
                {
                    player_to_attack.try_night_kill_single_attacker(
                        actor_ref,
                        game,
                        midnight_variables,
                        GraveKiller::Role(Role::Ambusher),
                        AttackPower::Basic,
                        false
                    );

                    for visitor in ambush_visit.target.all_direct_night_visitors_cloned(midnight_variables){
                        if visitor == player_to_attack || visitor == actor_ref {continue;}
                        visitor.push_night_message(midnight_variables, ChatMessageVariant::AmbusherCaught { ambusher: actor_ref });
                    }
                }
            }
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Ambusher, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Ambusher, 0),
            false
        )
    }
    fn default_insider_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
       vec![
           crate::game::components::insider_group::InsiderGroupID::Mafia
       ].into_iter().collect()
   }
}