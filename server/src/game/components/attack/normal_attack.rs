use crate::{
    game::{
        attack_power::AttackPower, chat::ChatMessageVariant, components::graves::grave::Grave, 
        player::PlayerReference, Game
    },
    vec_set::VecSet
};

pub struct Attack{
    send_messages: bool,
    defender: PlayerReference,
    attackers: VecSet<PlayerReference>,
    attack_power: AttackPower,
    grave: Grave
}
impl Attack{
    pub fn new_attack(
        game: &mut Game,
        send_messages: bool,
        defender: PlayerReference,
        attackers: VecSet<PlayerReference>,
        attack_power: AttackPower,
        grave: Grave
    )->bool{
        Self::new(send_messages, defender, attackers, attack_power, grave).attack(game)
    }
    fn new(
        send_messages: bool,
        defender: PlayerReference,
        attackers: VecSet<PlayerReference>,
        attack_power: AttackPower,
        grave: Grave
    )->Self{
        Self{
            send_messages,
            defender,
            attackers,
            attack_power,
            grave,
        }
    }
    fn attack(self, game: &mut Game)->bool{
        if self.defense_check(game) {return false;}
        self.defender.die_and_add_grave(game, self.grave);
        true
    }
    fn defense_check(
        &self,
        game: &mut Game,
    )->bool{
        let blocked = self.defender.normal_defense(game).can_block(self.attack_power);
        if !self.send_messages {return blocked};

        if blocked {
            self.defender.add_private_chat_message(game, ChatMessageVariant::YouSurvivedAttack);
            for attacker in self.attackers.iter() {
                attacker.add_private_chat_message(game, ChatMessageVariant::SomeoneSurvivedYourAttack);
            }
        }else{
            self.defender.add_private_chat_message(game, ChatMessageVariant::YouWereAttacked);
            for attacker in self.attackers.iter() {
                attacker.add_private_chat_message(game, ChatMessageVariant::YouAttackedSomeone);
            }
        }
        blocked
    }
}