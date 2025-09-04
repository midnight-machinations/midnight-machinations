use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind}, role_outline_reference::RoleOutlineReference, Game}, vec_set::VecSet};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TwoRoleOutlineOptionSelection(
    pub Option<RoleOutlineReference>,
    pub Option<RoleOutlineReference>
);

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AvailableTwoRoleOutlineOptionSelection(pub VecSet<Option<RoleOutlineReference>>);
impl AvailableSelectionKind for AvailableTwoRoleOutlineOptionSelection{
    type Selection = TwoRoleOutlineOptionSelection;
    fn validate_selection(&self, _game: &Game, selection: &TwoRoleOutlineOptionSelection)->bool{
        self.0.contains(&selection.0) && 
        self.0.contains(&selection.1) && 
        (selection.0.is_none() || selection.0 != selection.1)
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        TwoRoleOutlineOptionSelection(None, None)
    }
}

impl PartialOrd for AvailableTwoRoleOutlineOptionSelection{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}
impl Ord for AvailableTwoRoleOutlineOptionSelection{
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering{
        std::cmp::Ordering::Equal
    }
}


impl ControllerInput{
    pub fn get_two_role_outline_option_selection_if_id(&self, id: ControllerID)->Option<TwoRoleOutlineOptionSelection>{
        if id != self.id() {return None};
        let ControllerSelection::TwoRoleOutlineOption(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_two_role_outline_option_selection<'a>(&self, game: &'a Game)->Option<&'a TwoRoleOutlineOptionSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::TwoRoleOutlineOption(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}