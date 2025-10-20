use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant, event::on_midnight::{OnMidnightFold, OnMidnight, OnMidnightPriority}, player::PlayerReference, Game};

pub struct Guard;
impl Guard{
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority){
        if priority != OnMidnightPriority::Investigative {return};

        for player in PlayerReference::all_players(game){
            for protected in player.guarded_players(midnight_variables).clone() {
                if protected.night_attacked(midnight_variables){
                    player.push_night_message(midnight_variables, ChatMessageVariant::YouGuardedSomeone);
                    protected.push_night_message(midnight_variables, ChatMessageVariant::YouWereGuarded);
                }
            }
        }
    }
}
impl PlayerReference{
    pub fn guard_player(self, game: &mut Game, midnight_variables: &mut OnMidnightFold, guarded: PlayerReference){
        guarded.increase_defense_to(game, midnight_variables, DefensePower::Protected);
        midnight_variables.get_mut(self).guarded_players.push(guarded);
    }
    pub fn guarded_players(self, midnight_variables: &OnMidnightFold)->&Vec<PlayerReference>{
        &midnight_variables.get(self).guarded_players
    }
}