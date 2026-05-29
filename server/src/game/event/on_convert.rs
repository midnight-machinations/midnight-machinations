use crate::game::{event::EventData, player::PlayerReference};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnConvert{
    pub player: PlayerReference,
    pub old: crate::game::components::win_condition::WinCondition,
    pub new: crate::game::components::win_condition::WinCondition,
}
impl OnConvert{
    pub fn new(player: PlayerReference, old: crate::game::components::win_condition::WinCondition, new: crate::game::components::win_condition::WinCondition) -> (Self, ()){
        (Self{ player, old, new }, ())
    }
}
impl EventData for OnConvert{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnConvert{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::components::synopsis::SynopsisTracker::on_convert
    ]}
}