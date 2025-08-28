use crate::game::{
    components::{
        cult::Cult, enfranchise::Enfranchise, forfeit_vote::ForfeitNominationVote, mafia::Mafia
    },
    modifiers::ModifierSettings, player::PlayerReference, Game
};

#[must_use = "Event must be invoked"]
pub struct OnGameStart;
impl OnGameStart{
    pub fn invoke(game: &mut Game){
        ModifierSettings::on_game_start(game);

        for player in PlayerReference::all_players(game){
            player.on_game_start(game);
        }
        Mafia::on_game_start(game);
        Cult::on_game_start(game);
        Enfranchise::on_game_start(game);
        ForfeitNominationVote::on_game_start(game);
    }
}