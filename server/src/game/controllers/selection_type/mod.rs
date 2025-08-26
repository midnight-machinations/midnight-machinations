pub mod two_player_option_selection; pub use two_player_option_selection::*;
pub mod two_role_option_selection; pub use two_role_option_selection::*;
pub mod two_role_outline_option_selection; pub use two_role_outline_option_selection::*;
pub mod role_list_selection; pub use role_list_selection::*;
pub mod kira_selection; pub use kira_selection::*;
pub mod player_list_selection; pub use player_list_selection::*;
pub mod integer_selection; pub use integer_selection::*;
pub mod chat_message_selection; pub use chat_message_selection::*;

use serde::{Deserialize, Serialize};

use crate::game::{controllers::AvailableControllerSelection, Game};

use super::{controller_selection::ControllerSelection, ControllerInput, ControllerID};


pub trait AvailableSelectionKind: Into<AvailableControllerSelection>{
    type Selection: Into<ControllerSelection>;
    fn validate_selection(&self, game: &Game, selection: &Self::Selection)->bool;
    fn default_selection(&self, game: &Game) -> Self::Selection;
}


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitSelection;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AvailableUnitSelection;
impl AvailableSelectionKind for AvailableUnitSelection {
    type Selection = UnitSelection;

    fn validate_selection(&self, _game: &crate::game::Game, _selection: &Self::Selection)->bool {
        true
    }

    fn default_selection(&self, _game: &crate::game::Game) -> Self::Selection {
        UnitSelection
    }
}



#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BooleanSelection(pub bool);
impl ControllerInput{
    pub fn get_boolean_selection_if_id(&self, id: ControllerID)->Option<BooleanSelection>{
        if id != self.id() {return None};
        let ControllerSelection::Boolean(selection) = self.selection() else {return None};
        Some(selection)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AvailableBooleanSelection;
impl AvailableSelectionKind for AvailableBooleanSelection {
    type Selection = BooleanSelection;

    fn validate_selection(&self, _game: &crate::game::Game, _selection: &Self::Selection)->bool {
        true
    }

    fn default_selection(&self, _game: &crate::game::Game) -> Self::Selection {
        BooleanSelection(false)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringSelection(pub String);

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AvailableStringSelection;
impl AvailableSelectionKind for AvailableStringSelection {
    type Selection = StringSelection;

    fn validate_selection(&self, _game: &crate::game::Game, _selection: &Self::Selection)->bool {
        true
    }

    fn default_selection(&self, _game: &crate::game::Game) -> Self::Selection {
        StringSelection(String::new())
    }
}
impl ControllerInput{
    pub fn get_string_selection_if_id(&self, id: ControllerID)->Option<StringSelection>{
        if id != self.id() {return None};
        let ControllerSelection::String(selection) = self.selection() else {return None};
        Some(selection)
    }
}