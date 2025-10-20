use crate::game::{abilities_component::Abilities, components::mafia::Mafia, event::{on_midnight::OnMidnightFold, EventData}, player::PlayerReference};

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
}