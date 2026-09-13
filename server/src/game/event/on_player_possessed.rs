use crate::game::{abilities_component::Abilities, components::mafia::Mafia, event::{on_midnight::OnMidnightFold, EventData}, player::PlayerReference, Game};

pub struct OnPlayerPossessed{
    pub possessed: PlayerReference,
    pub possessed_into: PlayerReference,
}
impl OnPlayerPossessed{
    pub fn new(possessed: PlayerReference, possessed_into: PlayerReference)->Self{
        Self { possessed, possessed_into }
    }
}
impl EventData for OnPlayerPossessed {
    type FoldValue = OnMidnightFold;
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Abilities::on_player_possessed,
        Mafia::on_player_possessed
    ]}

    fn log(&self, _game: &Game, _fold: &OnMidnightFold) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "possessed": self.possessed, "possessedInto": self.possessed_into }))
    }
}