use crate::game::{
    components::{synopsis::SynopsisTracker, win_condition::WinCondition}, event::Event, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnConvert{
    pub player: PlayerReference,
    pub old: WinCondition,
    pub new: WinCondition,
}
impl OnConvert{
    pub fn new(player: PlayerReference, old: WinCondition, new: WinCondition) -> Self{
        Self{ player, old, new }
    }
}
impl Event for OnConvert{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![SynopsisTracker::on_convert]}
    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}