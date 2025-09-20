use rand::seq::SliceRandom;
use serde::Serialize;
use crate::game::components::night_visits::{NightVisitsIterator, Visits};
use crate::game::controllers::ControllerParametersMap;
use crate::game::attack_power::AttackPower;
use crate::game::chat::ChatMessageVariant;
use crate::game::components::graves::grave::GraveKiller;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;

use crate::game::role::informant::Informant;
use crate::game::visit::Visit;

use crate::game::Game;
use super::{common_role, ControllerID, Role, RoleStateTrait};


#[derive(Debug, Clone, Serialize, Default)]
pub struct Ojo;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Ojo {
    type ClientAbilityState = Ojo;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        match priority {
            OnMidnightPriority::Kill => {
                if game.day_number() == 1 {return}
                let actor_visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
                if let Some(visit) = actor_visits.first(){
                    let target_ref = visit.target;
            
                    target_ref.try_night_kill_single_attacker(
                        actor_ref,
                        game,
                        midnight_variables,
                        GraveKiller::Role(Role::Ojo),
                        AttackPower::Basic,
                        true
                    );
                }
            },
            OnMidnightPriority::Investigative => {
                PlayerReference::all_players(game)
                    .for_each(|player_ref|{

                    let mut players: Vec<PlayerReference> = Visits::into_iter(midnight_variables)
                        .with_visitor(player_ref)
                        .with_investigatable()
                        .map_target()
                        .collect();
                    players.shuffle(&mut rand::rng());

                    actor_ref.push_night_message(midnight_variables, 
                        ChatMessageVariant::WerewolfTrackingResult{
                            tracked_player: player_ref, 
                            players
                        }
                    );
                });
            }
            _ => ()
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> super::ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Ojo, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            .add_grayed_out_condition(game.day_number() <= 1)
            .build_map()  
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Ojo, 0),
            true
        )
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Ojo) || fold.cancelled {return}
        PlayerReference::all_players(game).for_each(|p|actor_ref.reveal_players_role(game, p));
    }
    fn on_conceal_role(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, concealed_player: PlayerReference){
        if player != actor_ref {return};

        actor_ref.reveal_players_role(game, concealed_player);
    }
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        Informant::read_whispers(false, game, actor_ref, event, fold, priority);
    }
}