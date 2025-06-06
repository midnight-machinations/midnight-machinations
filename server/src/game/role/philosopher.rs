use serde::Serialize;

use crate::game::ability_input::AvailableTwoPlayerOptionSelection;
use crate::game::components::confused::Confused;
use crate::game::components::win_condition::WinCondition;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;
use crate::vec_set;

use super::{common_role, ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Philosopher;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Philosopher {
    type ClientRoleState = Philosopher;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(first_visit) = actor_visits.get(0) else {return;};
        let Some(second_visit) = actor_visits.get(1) else {return;};

        let enemies = if Confused::is_confused(game, actor_ref) {
            false
        } else {
            Philosopher::players_are_enemies_night(game, midnight_variables, first_visit.target, second_visit.target)
        };

        let message = ChatMessageVariant::SeerResult{ enemies };
        
        actor_ref.push_night_message(midnight_variables, message);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {

        let available_players: vec_set::VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p| p.alive(game) && *p != actor_ref)
            .collect();

        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Philosopher, 0))
            .available_selection(AvailableTwoPlayerOptionSelection {
                available_first_players: available_players.clone(),
                available_second_players: available_players,
                can_choose_duplicates: false,
                can_choose_none: true,
            })
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Philosopher, 0),
            false
        )
    }
}
impl Philosopher{
    pub fn players_are_enemies_night(game: &Game, midnight_variables: &MidnightVariables, a: PlayerReference, b: PlayerReference) -> bool {
        if a.has_suspicious_aura(game, midnight_variables) || b.has_suspicious_aura(game, midnight_variables){
            true
        }else if a.has_innocent_aura(game) || b.has_innocent_aura(game){
            false
        }else{
            !WinCondition::are_friends(a.win_condition(game), b.win_condition(game))
        }
    }
}