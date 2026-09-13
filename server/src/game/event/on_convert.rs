use crate::game::{
    components::{synopsis::SynopsisTracker, win_condition::WinCondition}, event::EventData, player::PlayerReference, Game
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

    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "player": self.player,
            "old": self.old.clone(),
            "new": self.new.clone(),
        }))
    }
}