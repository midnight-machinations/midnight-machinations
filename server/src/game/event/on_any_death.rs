use crate::game::{
    abilities_component::Abilities,
    components::{
        cult::Cult, dead_can_still_play_message::DeadCanStillPlayMessage, mafia::Mafia
    },
    event::EventData, modifiers::ModifierSettings, player::PlayerReference,
};

#[must_use = "Event must be invoked"]
pub struct OnAnyDeath{
    pub dead_player: PlayerReference,
}
impl OnAnyDeath{
    pub fn new(dead_player: PlayerReference) -> (Self, ()){
        (Self{dead_player}, ())
    }
}
impl EventData for OnAnyDeath{
    type FoldValue = ();

    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Mafia::on_any_death,
        Cult::on_any_death,
        ModifierSettings::on_any_death,
        Abilities::on_any_death,
        DeadCanStillPlayMessage::on_any_death
    ]}
}