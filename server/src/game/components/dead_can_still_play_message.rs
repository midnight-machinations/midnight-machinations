use crate::game::{chat::ChatMessageVariant, event::on_any_death::OnAnyDeath, player::PlayerReference, role::Role, Game};

pub struct DeadCanStillPlayMessage;

impl DeadCanStillPlayMessage {
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        if
            PlayerReference::all_players(game)
                .any(|player|matches!(player.role(game), Role::Medium))
        {
            event.dead_player.add_private_chat_message(
                game,
                ChatMessageVariant::MediumExists
            );
        }
    }
}