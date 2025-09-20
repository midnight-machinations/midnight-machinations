use serde::Serialize;

use crate::game::components::graves::grave::Grave;
use crate::game::components::night_visits::Visits;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::win_condition::WinCondition;
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, Role, RoleStateTrait};


#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Scarecrow;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Scarecrow {
    type ClientAbilityState = Scarecrow;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        let Some(target) = Visits::default_target(game, midnight_variables, actor_ref) else {return};

        if matches!(priority, OnMidnightPriority::PreWard | OnMidnightPriority::Ward) {
            target.ward_night_action(game, midnight_variables, priority);
        }
        
        if matches!(priority, OnMidnightPriority::Ward) {
            actor_ref.reveal_players_role(game, target);
            actor_ref.push_night_message(
                midnight_variables, ChatMessageVariant::TargetHasRole { role: target.role(game) }
            );
        };
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Scarecrow, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Scarecrow, 0),
            false
        ).into_iter().map(|mut v|{v.wardblock_immune = true; v}).collect()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        if
            actor_ref.alive(game) &&
            PlayerReference::all_players(game)
                .filter(|p|p.alive(game))
                .filter(|p|p.keeps_game_running(game))
                .all(|p|
                    WinCondition::are_friends(p.win_condition(game), actor_ref.win_condition(game))
                )

        {
            actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
        }
    }
    fn on_visit_wardblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _visit: Visit) {}
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}