use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::{
    game::{
        controllers::{
            controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind
        },
        role::Role, Game
    },
    vec_set::VecSet
};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TwoRoleOptionSelection(pub Option<Role>, pub Option<Role>);
impl TwoRoleOptionSelection{
    pub fn any_in_common(&self, other: &TwoRoleOptionSelection) -> bool{
        (self.0.is_some() && self.0 == other.0) || 
        (self.0.is_some() && self.0 == other.1) || 
        (self.1.is_some() && self.1 == other.0) || 
        (self.1.is_some() && self.1 == other.1)
    }
    pub fn same_role(&self) -> bool{
        self.0.is_some() && self.0 == self.1 
    }
    pub fn contains(&self, role: Role) -> bool{
        self.0 == Some(role) || self.1 == Some(role)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableTwoRoleOptionSelection{
    pub available_roles: VecSet<Option<Role>>,
    
    pub can_choose_duplicates: bool
}
impl AvailableSelectionKind for AvailableTwoRoleOptionSelection{
    type Selection = TwoRoleOptionSelection;
    fn validate_selection(&self, _game: &Game, selection: &TwoRoleOptionSelection)->bool{
        if !self.can_choose_duplicates && selection.same_role(){
            return false
        }
        self.available_roles.contains(&selection.0) && self.available_roles.contains(&selection.1)
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        TwoRoleOptionSelection(None, None)
    }
}
impl PartialOrd for AvailableTwoRoleOptionSelection{
    fn partial_cmp(&self, other: &Self)->Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}
impl Ord for AvailableTwoRoleOptionSelection{
    fn cmp(&self, _other: &Self)->Ordering{
        Ordering::Equal
    }
}




impl ControllerInput{
    pub fn get_two_role_option_selection_if_id(&self, id: ControllerID)->Option<TwoRoleOptionSelection>{
        if id != self.id() {return None};
        let ControllerSelection::TwoRoleOption(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_two_role_option_selection<'a>(&self, game: &'a Game)->Option<&'a TwoRoleOptionSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::TwoRoleOption(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}