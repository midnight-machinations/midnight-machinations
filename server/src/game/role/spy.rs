use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::game::event::on_midnight::MidnightVariables;
use crate::game::{attack_power::DefensePower, event::on_midnight::OnMidnightPriority};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::player::PlayerReference;

use crate::game::visit::Visit;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Spy;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SpyBug{
    Silenced, 
    Roleblocked, Wardblocked,
    Protected, 
    Transported, Possessed
}


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateImpl for Spy {
    type ClientRoleState = Spy;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Investigative => {
                if actor_ref.night_blocked(midnight_variables) {return;}
                if actor_ref.ability_deactivated_from_death(game) {return;}

                let mut mafia_visits = vec![];
                for other_player in PlayerReference::all_players(game){
                    if !InsiderGroupID::Mafia.contains_player(game, other_player) {continue}
                    mafia_visits.append(
                        &mut other_player.tracker_seen_visits(game, midnight_variables)
                            .iter()
                            .map(|v|v.target.index())
                            .collect()
                    );
                }
                mafia_visits.shuffle(&mut rand::rng());
                
                actor_ref.push_night_message(midnight_variables, ChatMessageVariant::SpyMafiaVisit { players: mafia_visits });               
            },
            OnMidnightPriority::SpyBug => {
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                let Some(visit) = actor_visits.first() else {return};

                for message in visit.target.night_messages(midnight_variables).clone(){
                    if let Some(message) = match message{
                        ChatMessageVariant::Silenced => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Silenced }),
                        ChatMessageVariant::RoleBlocked => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Roleblocked }),
                        ChatMessageVariant::YouWereGuarded => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Protected }),
                        ChatMessageVariant::Transported => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Transported }),
                        ChatMessageVariant::YouWerePossessed { immune: _ } => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Possessed }),
                        ChatMessageVariant::Wardblocked => Some(ChatMessageVariant::SpyBug { bug: SpyBug::Wardblocked }),
                        _ => None
                    }{
                        actor_ref.push_night_message(midnight_variables, message);
                    }
                };
            }
            _=>{}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Spy, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            .add_grayed_out_condition(false)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Spy, 0),
            false
        )
    }
}