use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{ControllerSelection, AvailableSelectionKind, ControllerID}, player::PlayerReference, Game}, vec_set::VecSet};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TwoPlayerOptionSelection(pub Option<(PlayerReference, PlayerReference)>);
impl TwoPlayerOptionSelection{
    pub fn any_in_common(&self, other: &TwoPlayerOptionSelection) -> bool{
        match (self.0, other.0) {
            (Some((first, second)), Some((other_first, other_second))) => {
                first == other_first || 
                first == other_second || 
                second == other_first || 
                second == other_second
            },
            _ => false
        }
    }
    pub fn same_role(&self) -> bool{
        if let Some((first, second)) = self.0{
            first == second
        }else{
            false
        }
    }
    pub fn contains(&self, player: PlayerReference) -> bool{
        if let Some((first, second)) = self.0{
            first == player || second == player
        }else{
            false
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableTwoPlayerOptionSelection{
    pub available_first_players: VecSet<PlayerReference>,
    pub available_second_players: VecSet<PlayerReference>,
    
    pub can_choose_duplicates: bool,
    pub can_choose_none: bool
}
impl AvailableTwoPlayerOptionSelection{
    pub fn same_players(available_players: VecSet<PlayerReference>, can_choose_duplicates: bool, can_choose_none: bool) -> Self {
        Self { 
            available_first_players: available_players.clone(), 
            available_second_players: available_players, 
            can_choose_duplicates, 
            can_choose_none 
        }
    }
}
impl PartialOrd for AvailableTwoPlayerOptionSelection{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}
impl Ord for AvailableTwoPlayerOptionSelection{
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering{
        Ordering::Equal
    }
}
impl AvailableSelectionKind for AvailableTwoPlayerOptionSelection{
    type Selection = TwoPlayerOptionSelection;
    fn validate_selection(&self, _game: &Game, selection: &TwoPlayerOptionSelection)->bool{
        let Some((first, second)) = selection.0 else {
            return self.can_choose_none
        };

        if !self.can_choose_duplicates && first == second{
            return false
        }

        if 
            !self.available_first_players.contains(&first) || 
            !self.available_second_players.contains(&second)
        {
            return false
        }
        
        true
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        TwoPlayerOptionSelection(None)
    }
}

impl ControllerID{
    pub fn get_two_player_option_selection<'a>(&self, game: &'a Game)->Option<&'a TwoPlayerOptionSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::TwoPlayerOption(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}