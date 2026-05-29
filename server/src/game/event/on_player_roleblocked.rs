use crate::game::{event::EventData, player::PlayerReference};
use super::{EventListenerFunction, LegacyEventData, on_midnight::OnMidnightFold};

#[must_use = "Event must be invoked"]
pub struct OnPlayerRoleblocked{
    pub player: PlayerReference,
    pub invisible: bool,
}
impl OnPlayerRoleblocked{
    pub fn new(player: PlayerReference, invisible: bool) -> Self{
        Self{player, invisible}
    }
}
impl EventData for OnPlayerRoleblocked{
    type FoldValue = OnMidnightFold;
}
#[allow(deprecated)]
impl LegacyEventData for OnPlayerRoleblocked{
    type FoldValue = OnMidnightFold;
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::abilities_component::Abilities::on_player_roleblocked,
        crate::game::components::mafia::Mafia::on_player_roleblocked,
        crate::game::components::blocked::BlockedComponent::on_player_roleblocked,
    ]}
}