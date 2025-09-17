use crate::game::{
    game_conclusion::GameConclusion, Game
};

#[must_use = "Event must be invoked"]
pub struct OnGameEnding {
    conclusion: GameConclusion
}

impl OnGameEnding{
    pub fn new(conclusion: GameConclusion) -> Self {
        OnGameEnding {
            conclusion
        }
    }
    pub fn invoke(&self, game: &mut Game){
        game.on_game_ending(self.conclusion);
    }
}
