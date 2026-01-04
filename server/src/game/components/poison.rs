use crate::game::{
    Game, chat::ChatMessageVariant, components::attack::night_attack::NightAttack, event::on_midnight::{OnMidnight, OnMidnightFold, OnMidnightPriority}, player::PlayerReference,
};

impl Game {
    pub fn poison(&self)->&Poison{
        &self.poison
    }
    pub fn set_poison(&mut self, poison: Poison){
        self.poison = poison;
    }
}

#[derive(Default, Clone)]
pub struct Poison{
    poisons: Vec<PlayerPoison>
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerPoison {
    player: PlayerReference,
    attack: NightAttack,
}

#[derive(PartialEq, Eq)]
pub enum PoisonAlert {
    NoAlert,
    Alert
}

impl Poison{
    pub fn poison_player(
        game: &mut Game,
        midnight_variables: &mut OnMidnightFold,
        target: PlayerReference,
        attack: NightAttack,
        alert: PoisonAlert,
    ){
        let mut poison = game.poison().clone();
        poison.poisons.push(PlayerPoison { player: target, attack });

        if alert == PoisonAlert::Alert {
            for poison in &poison.poisons {
                poison.player.push_night_message(midnight_variables, ChatMessageVariant::YouArePoisoned);
            }
        }

        game.set_poison(poison);
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority){
        if priority != OnMidnightPriority::Kill{ return; }

        let mut poison = game.poison().clone();

        for poison in &mut poison.poisons {
            poison.attack.attack(game, midnight_variables, poison.player);
        }
        poison.poisons.clear();
        
        game.set_poison(poison);
    }
}