use crate::game::{
    game_conclusion::GameConclusion, player::PlayerReference, Game
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
        for player_ref in PlayerReference::all_players(game){
            player_ref.on_game_ending(game);
        }

        game.on_game_ending(self.conclusion);
    }
}
