pub mod normal_attack;
pub mod night_attack;

use crate::{game::{components::attack::night_attack::NightAttack, prelude::*}, vec_set::VecSet};

impl PlayerReference{
    #[expect(clippy::too_many_arguments, reason="this function is goated tho")]
    pub fn rampage(
        &self, game: &mut Game,
        midnight_variables: &mut OnMidnightFold,
        attacker: PlayerReference,
        grave_killer: GraveKiller,
        attack: AttackPower,
        should_leave_death_note: bool,
        filter_visit: impl FnMut(&Visit) -> bool
    ){
        Visits::into_iter(midnight_variables)
            .filter(filter_visit)
            .with_target(*self)
            .with_direct()
            .map_visitor()
            .for_each(|p|{
                p.try_night_kill_single_attacker(
                    attacker,
                    game,
                    midnight_variables,
                    grave_killer.clone(),
                    attack,
                    should_leave_death_note
                );
            });
    }


    /// Returns true if attack overpowered defense
    pub fn try_night_kill_single_attacker(&self, attacker_ref: PlayerReference, game: &mut Game, midnight_variables: &mut OnMidnightFold, grave_killer: GraveKiller, attack: AttackPower, should_leave_death_note: bool) -> bool {
        self.try_night_kill(
            vec![attacker_ref].into_iter().collect(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            should_leave_death_note
        )
    }
    pub fn try_night_kill(&self, attackers: VecSet<PlayerReference>, game: &mut Game, midnight_variables: &mut OnMidnightFold, grave_killer: GraveKiller, attack_power: AttackPower, leave_death_note: bool) -> bool {
        NightAttack::new_attack(game, midnight_variables, true, *self, attackers, attack_power, leave_death_note, grave_killer)
    }
    pub fn try_night_kill_no_attacker(&self, game: &mut Game, midnight_variables: &mut OnMidnightFold, grave_killer: GraveKiller, attack: AttackPower) -> bool {
        self.try_night_kill(
            VecSet::new(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            false
        )
    }
}