use crate::{game::{Game, attack_power::AttackPower, chat::ChatMessageVariant, components::graves::grave::GraveKiller, event::on_midnight::OnMidnightFold, player::PlayerReference, prelude::{NightVisitsIterator, Visit, Visits}}, vec_set::VecSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NightAttack<GK = GraveKiller> {
    send_messages: bool,
    attackers: VecSet<PlayerReference>,
    attack_power: AttackPower,
    leave_death_note: bool,
    grave_killer: GK
}

impl NightAttack<()> {
    pub fn new() -> Self {
        NightAttack {
            send_messages: true,
            attackers: VecSet::new(),
            attack_power: AttackPower::default(),
            leave_death_note: false,
            grave_killer: (),
        }
    }
}

impl<G> NightAttack<G> {
    pub fn hide_messages(mut self) -> Self {
        self.send_messages = false;
        self
    }
    pub fn attackers(mut self, attackers: impl IntoIterator<Item = PlayerReference>) -> Self {
        self.attackers = attackers.into_iter().collect();
        self
    }
    pub fn power(mut self, attack_power: AttackPower) -> Self {
        self.attack_power = attack_power;
        self
    }
    pub fn leave_death_note(mut self) -> Self {
        self.leave_death_note = true;
        self
    }
}

impl NightAttack<()> {
    pub fn grave_killer(self, grave_killer: impl Into<GraveKiller>) -> NightAttack<GraveKiller> {
        NightAttack {
            send_messages: self.send_messages,
            attackers: self.attackers,
            attack_power: self.attack_power,
            leave_death_note: self.leave_death_note,
            grave_killer: grave_killer.into(),
        }
    }
}

impl NightAttack<GraveKiller> {
    pub fn rampage(self, game: &mut Game, fold: &mut OnMidnightFold, target: PlayerReference, filter_visit: impl FnMut(&Visit) -> bool) {
        Visits::into_iter(fold)
            .filter(filter_visit)
            .with_target(target)
            .with_direct()
            .map_visitor()
            .for_each(|p|{
                NightAttack {
                    send_messages: self.send_messages,
                    attackers: self.attackers.clone(),
                    attack_power: self.attack_power,
                    leave_death_note: self.leave_death_note,
                    grave_killer: self.grave_killer.clone(),
                }.attack(game, fold, p);
            });
    }

    pub fn attack(&self, game: &mut Game, fold: &mut OnMidnightFold, defender: PlayerReference)->bool{

        defender.set_night_attacked(fold, true);
        if self.defense_check(game, fold, defender) {return false;}

        defender.push_night_grave_killers(fold, self.grave_killer.clone());

        if self.leave_death_note {
            for attacker in self.attackers.iter() {
                if let Some(note) = attacker.death_note(game) {
                    defender.push_night_grave_death_notes(fold, note.clone());
                }
            }
        }

        if !defender.alive(game) { return true }

        defender.set_night_died(fold, true);

        true
    }

    pub fn defense_check(
        &self,
        game: &mut Game,
        fold: &mut OnMidnightFold,
        defender: PlayerReference
    )->bool{
        let blocked = defender.night_defense(game, fold).can_block(self.attack_power);
        if !self.send_messages {return blocked};

        if blocked {
            defender.push_night_message(fold, ChatMessageVariant::YouSurvivedAttack);
            for attacker in self.attackers.iter() {
                attacker.push_night_message(fold, ChatMessageVariant::SomeoneSurvivedYourAttack);
            }
        }else{
            defender.push_night_message(fold, ChatMessageVariant::YouWereAttacked);
            for attacker in self.attackers.iter() {
                attacker.push_night_message(fold, ChatMessageVariant::YouAttackedSomeone);
            }
        }
        blocked
    }
}