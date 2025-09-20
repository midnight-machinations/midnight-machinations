use crate::game::{
    abilities_component::Abilities,
    components::{
        cult::Cult, dead_can_still_play_message::DeadCanStillPlayMessage, mafia::Mafia
    },
    event::Event, modifiers::ModifierSettings, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnAnyDeath{
    pub dead_player: PlayerReference,
}
impl OnAnyDeath{
    pub fn new(dead_player: PlayerReference) -> Self{
        Self{dead_player}
    }
}
impl Event for OnAnyDeath{
    type FoldValue = ();

    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Mafia::on_any_death,
        Cult::on_any_death,
        ModifierSettings::on_any_death,
        Abilities::on_any_death,
        DeadCanStillPlayMessage::on_any_death
    ]}

    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}