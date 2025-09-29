use crate::{game::{attack_power::AttackPower, chat::ChatMessageVariant, components::graves::grave::GraveKiller, event::on_midnight::MidnightVariables, player::PlayerReference, Game}, vec_set::VecSet};

pub struct NightAttack{
    send_messages: bool,
    defender: PlayerReference,
    attackers: VecSet<PlayerReference>,
    attack_power: AttackPower,
    leave_death_note: bool,
    grave_killer: GraveKiller
}
impl NightAttack{
    #[expect(clippy::too_many_arguments, reason="this function is goated tho")]
    pub fn new_attack(
        game: &mut Game,
        fold: &mut MidnightVariables,
        send_messages: bool,
        defender: PlayerReference,
        attackers: VecSet<PlayerReference>,
        attack_power: AttackPower,
        leave_death_note: bool,
        grave_killer: GraveKiller,
    )->bool{
        Self::new(send_messages, defender, attackers, attack_power, leave_death_note, grave_killer).attack(game, fold)
    }
    fn new(
        send_messages: bool,
        defender: PlayerReference,
        attackers: VecSet<PlayerReference>,
        attack_power: AttackPower,
        leave_death_note: bool,
        grave_killer: GraveKiller,
    )->Self{Self{
        send_messages,
        defender,
        attackers,
        attack_power,
        leave_death_note,
        grave_killer,
    }}
    fn attack(self, game: &mut Game, fold: &mut MidnightVariables)->bool{

        self.defender.set_night_attacked(fold, true);
        if self.defense_check(game, fold) {return false;}

        self.defender.push_night_grave_killers(fold, self.grave_killer);
            
        if self.leave_death_note {
            for attacker in self.attackers.iter() {
                if let Some(note) = attacker.death_note(game) {
                    self.defender.push_night_grave_death_notes(fold, note.clone());
                }
            }
        }

        if !self.defender.alive(game) { return true }

        self.defender.set_night_died(fold, true);

        true
    }

    fn defense_check(
        &self,
        game: &mut Game,
        fold: &mut MidnightVariables,
    )->bool{
        let blocked = self.defender.night_defense(game, fold).can_block(self.attack_power);
        if !self.send_messages {return blocked};

        if blocked {
            self.defender.push_night_message(fold, ChatMessageVariant::YouSurvivedAttack);
            for attacker in self.attackers.iter() {
                attacker.push_night_message(fold, ChatMessageVariant::SomeoneSurvivedYourAttack);
            }
        }else{
            self.defender.push_night_message(fold, ChatMessageVariant::YouWereAttacked);
            for attacker in self.attackers.iter() {
                attacker.push_night_message(fold, ChatMessageVariant::YouAttackedSomeone);
            }
        }
        blocked
    }
}