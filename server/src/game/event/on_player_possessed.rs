use crate::game::{event::EventData, player::PlayerReference};
use super::{EventListenerFunction, LegacyEventData};

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
    type FoldValue = crate::game::event::on_midnight::OnMidnightFold;
}
#[allow(deprecated)]
impl LegacyEventData for OnPlayerPossessed {
    type FoldValue = crate::game::event::on_midnight::OnMidnightFold;
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::abilities_component::Abilities::on_player_possessed,
        crate::game::components::mafia::Mafia::on_player_possessed
    ]}
}