use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind}, player::PlayerReference, Game}, vec_set::VecSet};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerListSelection(pub Vec<PlayerReference>);

impl PlayerListSelection {
    /// Meant for setting default selections
    /// if the option is none, it returns an empty selection
    pub fn one(player: Option<PlayerReference>) -> Self {
        player.map_or_else(Self::default, |p| Self(vec![p]))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailablePlayerListSelection{
    pub available_players: VecSet<PlayerReference>,
    pub can_choose_duplicates: bool,
    pub max_players: Option<u8>
}
impl AvailableSelectionKind for AvailablePlayerListSelection{
    type Selection = PlayerListSelection;
    fn validate_selection(&self, _game: &Game, selection: &PlayerListSelection)->bool{
        self.available_players.is_superset(&selection.0.iter().copied().collect()) && 
        (self.can_choose_duplicates || selection.0.len() == selection.0.iter().collect::<Vec<_>>().len()) &&
        self.max_players.is_none_or(|max| selection.0.len() <= max as usize)
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        PlayerListSelection(Vec::new())
    }
}


impl ControllerInput{
    pub fn get_player_list_selection_if_id(&self, id: ControllerID)->Option<PlayerListSelection>{
        if id != self.id() {return None};
        let ControllerSelection::PlayerList(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_player_list_selection<'a>(&self, game: &'a Game)->Option<&'a PlayerListSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::PlayerList(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}