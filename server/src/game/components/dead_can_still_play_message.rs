use crate::game::{abilities_component::{ability_id::AbilityID, Abilities}, chat::ChatMessageVariant, event::on_any_death::OnAnyDeath, role::Role, Game};

pub struct DeadCanStillPlayMessage;

impl DeadCanStillPlayMessage {
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, _fold: &mut (), _priority: ()) {
        if
            !game.settings.enabled_roles.contains(&Role::Medium) &&
            !game.settings.enabled_roles.contains(&Role::Lich)
        {
            return
        }

        if
            Abilities::ids(game)
                .into_iter()
                .any(|id|{
                    matches!(id,
                        AbilityID::Role { role: Role::Medium, .. }|
                        AbilityID::Role { role: Role::Intern, .. }|
                        AbilityID::Role { role: Role::Tailor, .. }|
                        AbilityID::Role { role: Role::Employer, .. }|
                        AbilityID::Role { role: Role::TownCommonWildcard, .. }|
                        AbilityID::Role { role: Role::Wildcard, .. }|
                        AbilityID::Role { role: Role::TrueWildcard, .. }|
                        AbilityID::Role { role: Role::Lich, .. }|
                        AbilityID::Role { role: Role::FiendsWildcard, .. }
                    )
                })
            
        {
            event.dead_player.add_private_chat_message(
                game,
                ChatMessageVariant::MediumExists
            );
        }
    }
}