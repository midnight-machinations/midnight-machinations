
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::game::attack_power::AttackPower;
use crate::game::attack_power::DefensePower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::components::night_visits::NightVisits;
use crate::game::event::on_midnight::MidnightVariables;
use crate::game::event::on_midnight::OnMidnightPriority;
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;

use crate::game::Game;
use super::{
    ControllerID, ControllerParametersMap,
    Role, RoleStateImpl
};



#[derive(Clone, Debug, Default, Serialize)]
pub struct Ambusher;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Ambusher {
    type ClientRoleState = Ambusher;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}

        match priority {
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
                        visit.visitor.win_condition(game).is_loyalist_for(GameConclusion::Town)
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
                        GraveKiller::Role(Role::Ambusher),
                        AttackPower::Basic,
                        false
                    );

                    for visitor in target_ref.all_night_visitors_cloned(midnight_variables){
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
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
       vec![
           crate::game::components::insider_group::InsiderGroupID::Mafia
       ].into_iter().collect()
   }
}