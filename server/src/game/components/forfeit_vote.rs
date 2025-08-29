use crate::{
    game::{
        controllers::*, modifiers::ModifierID,
        phase::PhaseType, player::PlayerReference, Game
    },
    vec_set::VecSet
};

use super::tags::{TagSetID, Tags};

pub struct ForfeitNominationVote;
impl ForfeitNominationVote{
    pub fn controller_parameters_map(game: &Game)->ControllerParametersMap {
        if
            !game.modifier_settings().is_enabled(ModifierID::ForfeitNominationVote)
        {
            return ControllerParametersMap::default();
        }

        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|player|
                    ControllerParametersMap::builder(game)
                        .id(ControllerID::forfeit_vote(player))
                        .available_selection(AvailableBooleanSelection)
                        .add_grayed_out_condition(!player.alive(game) || game.current_phase().phase() != PhaseType::Discussion)
                        .reset_on_phase_start(PhaseType::Obituary)
                        .allow_players([player])
                        .build_map()
                )
        )
    }

    /// Must go before saved_controllers on phase start
    pub fn on_phase_start(game: &mut Game, phase: PhaseType){
        match phase {
            PhaseType::Nomination => {
                for player in PlayerReference::all_players(game){
                    if 
                        Self::player_chose_forfeit(game, player) && player.alive(game)
                    {
                        Tags::add_tag(game, TagSetID::ForfeitNominationVote, player);
                    }
                }
            },
            PhaseType::Dusk => {
                Tags::set_tagged(game, TagSetID::ForfeitNominationVote, &VecSet::new());
            },
            _ => {}
        }
    }

    pub fn on_game_start(game: &mut Game){
        Tags::set_viewers(game, TagSetID::ForfeitNominationVote, &PlayerReference::all_players(game).collect());
    }

    pub fn forfeited_vote(game: &Game, player: PlayerReference)->bool{
        Tags::has_tag(game, TagSetID::ForfeitNominationVote, player)
    }

    fn player_chose_forfeit(game: &Game, player: PlayerReference)->bool{
        matches!(
            ControllerID::forfeit_vote(player).get_boolean_selection(game),
            Some(BooleanSelection(true))
        )
    }
}