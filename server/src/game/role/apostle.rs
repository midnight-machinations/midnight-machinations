use serde::Serialize;

use crate::game::controllers::ControllerParametersMap;
use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::cult::{Cult, CultAbility};
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::components::win_condition::WinCondition;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;
use crate::game::game_conclusion::GameConclusion;

use crate::game::role_list::RoleSet;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{common_role, ControllerID, Role, RoleStateImpl};


#[derive(Clone, Debug, Default, Serialize)]
pub struct Apostle;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Apostle {
    type ClientRoleState = Apostle;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {

        match (priority, Cult::next_ability(game)) {
            (OnMidnightPriority::Kill, CultAbility::Kill) if game.cult().ordered_cultists.len() == 1 => {

                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actor_visits.first() else {return};
                let target_ref = visit.target;
                
                if target_ref.try_night_kill_single_attacker(
                    actor_ref, game, midnight_variables, GraveKiller::RoleSet(RoleSet::Cult), AttackPower::Basic, false
                ) {
                    Cult::set_ability_used_last_night(game, Some(CultAbility::Kill));
                }
            }
            (OnMidnightPriority::Convert, CultAbility::Convert) => {
                
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actor_visits.first() else {return};
                let target_ref = visit.target;

                if target_ref.night_defense(game, midnight_variables).can_block(AttackPower::Basic) {
                    actor_ref.push_night_message(midnight_variables, ChatMessageVariant::YourConvertFailed);
                    return
                }

                target_ref.set_night_convert_role_to(midnight_variables, Some(Role::Zealot.new_state(game)));
                InsiderGroupID::Cult.add_player_to_revealed_group(game, target_ref);
                target_ref.set_win_condition(game, WinCondition::new_loyalist(GameConclusion::Cult));
                
                Cult::set_ability_used_last_night(game, Some(CultAbility::Convert));
            }
            _ => {}
        }
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref, 
            ControllerID::role(actor_ref, Role::Apostle, 0),
            true
        )
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Apostle, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(
                game.cult().ordered_cultists.len() != 1 &&
                Cult::next_ability(game) == CultAbility::Kill
            )
            .build_map()
    }
    fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Cult
        ].into_iter().collect()
    }
}
