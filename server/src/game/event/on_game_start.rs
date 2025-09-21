use crate::game::{
    components::{
        cult::Cult, enfranchise::Enfranchise, forfeit_vote::ForfeitNominationVote, mafia::Mafia
    }, controllers::Controllers, event::Event, modifiers::ModifierSettings, Game
};

#[must_use = "Event must be invoked"]
pub struct OnGameStart;
impl OnGameStart{
    pub fn new()->Self{Self}
}
impl Event for OnGameStart{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        ModifierSettings::on_game_start,
        Mafia::on_game_start,
        Cult::on_game_start,
        Enfranchise::on_game_start,
        ForfeitNominationVote::on_game_start,
        Controllers::on_game_start,
    ]}
    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}