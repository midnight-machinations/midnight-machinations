use crate::{game::{
    attack_power::AttackPower, chat::ChatMessageVariant, components::graves::grave::GraveKiller, event::on_midnight::{OnMidnightFold, OnMidnight, OnMidnightPriority}, player::PlayerReference, Game
}, vec_set::VecSet};

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
struct PlayerPoison{
    player: PlayerReference,
    attack_power: AttackPower,
    grave_killer: GraveKiller,
    attackers: VecSet<PlayerReference>,
    leave_death_note: bool,
}
impl PlayerPoison{
    pub fn new(
        player: PlayerReference,
        attack_power: AttackPower,
        grave_killer: GraveKiller,
        attackers: VecSet<PlayerReference>,
        leave_death_note: bool,
    )->Self{
        Self{
            player,
            attack_power,
            grave_killer,
            attackers,
            leave_death_note,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum PoisonAlert {
    NoAlert,
    Alert
}

impl Poison{
    /// run this at night
    #[expect(clippy::too_many_arguments, reason = "This will be addressed in a future PR")]
    pub fn poison_player(
        game: &mut Game,
        midnight_variables: &mut OnMidnightFold,
        player: PlayerReference,
        attack_power: AttackPower,
        grave_killer: GraveKiller,
        attackers: VecSet<PlayerReference>,
        death_note: bool,
        alert: PoisonAlert,
    ){
        let mut poison = game.poison().clone();
        poison.poisons.push(PlayerPoison::new(
            player, attack_power, grave_killer, attackers, death_note
        ));

        if alert == PoisonAlert::Alert {
            for poison in poison.poisons.iter(){
                poison.player.push_night_message(midnight_variables, ChatMessageVariant::YouArePoisoned);
            }
        }

        game.set_poison(poison);
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority){
        if priority != OnMidnightPriority::Kill{ return; }

        let mut poison = game.poison().clone();

        for poison in poison.poisons.iter_mut(){
            Self::attack_poisoned_player(game, midnight_variables, poison.clone());
        }
        poison.poisons.clear();
        
        game.set_poison(poison);
    }
    fn attack_poisoned_player(game: &mut Game, midnight_variables: &mut OnMidnightFold, poison: PlayerPoison){
        poison.player.try_night_kill(
            poison.attackers.clone(),
            game,
            midnight_variables,
            poison.grave_killer,
            poison.attack_power,
            poison.leave_death_note
        );
    }
}