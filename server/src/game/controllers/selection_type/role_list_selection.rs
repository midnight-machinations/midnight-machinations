use serde::{Deserialize, Serialize};

use crate::{game::{controllers::{controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind}, role::Role, Game}, vec_set::VecSet};


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoleListSelection(pub Vec<Role>);


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableRoleListSelection{
    pub available_roles: VecSet<Role>,
    pub can_choose_duplicates: bool,
    pub max_roles: Option<u8>
}
impl AvailableSelectionKind for AvailableRoleListSelection{
    type Selection = RoleListSelection;
    fn validate_selection(&self, _game: &Game, selection: &RoleListSelection)->bool{
        self.available_roles.is_superset(&selection.0.iter().copied().collect()) && 
        (self.can_choose_duplicates || selection.0.len() == selection.0.iter().collect::<Vec<_>>().len()) &&
        self.max_roles.is_none_or(|max| selection.0.len() <= max as usize)
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        RoleListSelection(vec![])
    }
}


impl ControllerInput{
    pub fn get_role_list_selection_if_id(&self, id: ControllerID)->Option<RoleListSelection>{
        if id != self.id() {return None};
        let ControllerSelection::RoleList(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_role_list_selection<'a>(&self, game: &'a Game)->Option<&'a RoleListSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::RoleList(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
    pub fn get_role_list_selection_first<'a>(&self, game: &'a Game)->Option<&'a Role>{
        let roles = self.get_role_list_selection(game)?;
        roles.0.first()
    }
}