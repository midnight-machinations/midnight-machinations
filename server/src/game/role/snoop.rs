use serde::Serialize;

use crate::game::components::confused::Confused;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Snoop;

impl RoleStateImpl for Snoop {
    type ClientRoleState = Snoop;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}


        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        if let Some(visit) = actor_visits.first(){

            let townie = if Confused::is_confused(game, actor_ref) {
                Snoop::confused_result()
            }else{
                Snoop::result(game, midnight_variables, visit)
            };

            actor_ref.push_night_message(midnight_variables, 
                ChatMessageVariant::SnoopResult { townie }
            );
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Snoop, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Snoop, 0),
            false
        )
    }
}

impl Snoop{
    /// Is a town loyalist
    fn result(game: &Game, midnight_variables: &MidnightVariables, visit: &Visit)->bool{
        visit.target.win_condition(game).is_loyalist_for(GameConclusion::Town) &&
        !visit.target.has_suspicious_aura(game, midnight_variables) &&
        !Self::too_many_visitors(game, midnight_variables, visit)
    }
    fn confused_result()->bool{
        false
    }
    fn too_many_visitors(game: &Game, midnight_variables: &MidnightVariables, visit: &Visit)->bool{
        visit.visitor
            .all_night_visitors_cloned(midnight_variables)
            .iter()
            .map(|visitor|
                if visitor
                    .win_condition(game)
                    .is_loyalist_for(GameConclusion::Town)
                {    
                    0.5
                } else {
                    1.0
                }
            )
            .fold(0.0, |acc, visitor|acc + visitor) >= 1.0
    }
}