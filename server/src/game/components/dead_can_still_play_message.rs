use crate::game::{abilities_component::{ability_id::AbilityID, Abilities}, chat::ChatMessageVariant, event::on_any_death::OnAnyDeath, role::Role, Game};

pub struct DeadCanStillPlayMessage;

impl DeadCanStillPlayMessage {
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        if
            Abilities::ids(game)
                .into_iter()
                .any(|id|{
                    matches!(id, AbilityID::Role { role: Role::Medium, .. })
                })
        {
            event.dead_player.add_private_chat_message(
                game,
                ChatMessageVariant::MediumExists
            );
        }
    }
}