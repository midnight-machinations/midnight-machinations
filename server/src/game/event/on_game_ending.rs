use crate::game::{event::EventData, game_conclusion::GameConclusion, Game};
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnGameEnding {
    pub(crate) conclusion: GameConclusion
}
impl OnGameEnding{
    pub fn new(conclusion: GameConclusion) -> (Self, ()) {
        (Self {conclusion}, ())
    }
}
impl EventData for OnGameEnding{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnGameEnding{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        Game::on_game_ending,
    ]}
}