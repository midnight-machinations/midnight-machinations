use crate::game::{
    components::{synopsis::SynopsisTracker, win_condition::WinCondition}, event::EventData, player::PlayerReference
};

#[must_use = "Event must be invoked"]
pub struct OnConvert{
    pub player: PlayerReference,
    pub old: WinCondition,
    pub new: WinCondition,
}
impl OnConvert{
    pub fn new(player: PlayerReference, old: WinCondition, new: WinCondition) -> (Self, ()){
        (Self{ player, old, new }, ())
    }
}
impl EventData for OnConvert{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![SynopsisTracker::on_convert]}
}