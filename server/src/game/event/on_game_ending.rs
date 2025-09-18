use crate::game::{
    event::Event, game_conclusion::GameConclusion, Game
};

#[must_use = "Event must be invoked"]
pub struct OnGameEnding {
    pub(crate) conclusion: GameConclusion
}

impl OnGameEnding{
    pub fn new(conclusion: GameConclusion) -> Self {
        Self {
            conclusion
        }
    }
}
impl Event for OnGameEnding{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Game::on_game_ending,
    ]}
    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}