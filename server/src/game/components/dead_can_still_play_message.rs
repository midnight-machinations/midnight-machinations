use crate::game::{chat::ChatMessageVariant, player::PlayerReference, role::Role, Game};

pub struct DeadCanStillPlayMessage;

impl DeadCanStillPlayMessage {
    pub fn on_any_death(game: &mut Game, dead_player_ref: PlayerReference) {
        if
            PlayerReference::all_players(game).any(|player|
                matches!(player.role(game), Role::Medium | Role::Puppeteer)
            )
        {
            dead_player_ref.add_private_chat_message(
                game,
                ChatMessageVariant::MediumExists
            );
        }
    }
}