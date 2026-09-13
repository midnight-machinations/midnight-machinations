use crate::game::{
    components::{ascend::Ascend, hide_votes_message::HideVotesMessage}, controllers::Controllers, event::EventData, Game
};

pub struct OnTick;

impl OnTick{
    pub fn new()->(Self, ()){
        (Self{}, ())
    }
}
impl EventData for OnTick{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Controllers::on_tick,
        Ascend::on_tick,
        HideVotesMessage::on_tick
    ]}

    /// Probably don't need to log this, but keeping it for consistency with the other events? feel free 2 delete
    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        None
    }
}