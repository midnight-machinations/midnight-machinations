use serde::Serialize;

use crate::game::attack_power::DefensePower;
use crate::game::components::aura::Aura;
use crate::game::components::confused::Confused;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{chat::ChatMessageVariant, components::verdicts_today::VerdictsToday};
use crate::game::game_conclusion::GameConclusion;
use crate::game::player::PlayerReference;

use crate::game::Game;

use super::RoleStateTrait;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct TallyClerk;


impl RoleStateTrait for TallyClerk {
    type ClientAbilityState = TallyClerk;
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if actor_ref.night_blocked(midnight_variables) {return}
        if actor_ref.ability_deactivated_from_death(game) {return}
        if priority != OnMidnightPriority::Investigative {return;}

        let evil_count = if Confused::is_confused(game, actor_ref){
            Self::confused_result(game, midnight_variables)
        }else{
            Self::result(game, midnight_variables)
        };
        
        let message = ChatMessageVariant::TallyClerkResult{ evil_count };
        actor_ref.push_night_message(midnight_variables, message);
    }
}

impl TallyClerk {
    fn result(game: &Game, midnight_variables: &MidnightVariables)->u8{
        let mut out: u8 = 0;
        for player in PlayerReference::all_players(game)
            .filter(|player|player.alive(game))
            .filter(|player|VerdictsToday::player_guiltied_today(game, player))
        {
            if TallyClerk::player_is_suspicious(game, midnight_variables, player){
                out = out.saturating_add(1);
            }
        }
        out
    }
    fn confused_result(game: &Game, midnight_variables: &MidnightVariables)->u8{
        let total_guilties = VerdictsToday::guilties(game).len();

        let evil_count = Self::result(game, midnight_variables).saturating_add_signed(rand::random_range(0..=1));
        
        evil_count.min(total_guilties.try_into().unwrap_or(u8::MAX))
    }
    fn player_is_suspicious(game: &Game, midnight_variables: &MidnightVariables, player_ref: PlayerReference) -> bool {

        if Aura::suspicious(game, midnight_variables, player_ref){
            true
        }else if Aura::innocent(game, midnight_variables, player_ref){
            false
        }else{
            !player_ref.win_condition(game).is_loyalist_for(GameConclusion::Town)
        }
    }
}