use crate::game::{ability_input::{AvailableStringSelection, ControllerID, ControllerParametersMap, StringSelection}, player::PlayerReference, Game};

pub struct Alibi;
impl Alibi{
    pub fn controller_parameters_map(game: &Game)->ControllerParametersMap{
        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|p|Self::one_player_controller_parameters_map(game, p))
        )
    }
    fn one_player_controller_parameters_map(game: &Game, player: PlayerReference)->ControllerParametersMap{
        ControllerParametersMap::builder(game)
            .id(ControllerID::Alibi{player})
            .available_selection(AvailableStringSelection)
            .allow_players([player])
            .build_map()
    }
}
impl PlayerReference{
    pub fn alibi<'a>(&self, game: &'a Game) -> &'a str {
        let Some(StringSelection(string)) = 
            ControllerID::Alibi { player: *self }.get_string_selection(game) else {return ""};
        string
    }
}