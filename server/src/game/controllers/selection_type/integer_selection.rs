use serde::{Deserialize, Serialize};

use crate::game::{
    controllers::{
        controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind
    },
    Game
};


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntegerSelection(pub i8);


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AvailableIntegerSelection{
    pub min: i8,    //inclusive
    pub max: i8
}
impl AvailableSelectionKind for AvailableIntegerSelection{
    type Selection = IntegerSelection;
    fn validate_selection(&self, _game: &Game, selection: &IntegerSelection)->bool{
        selection.0 >= self.min && selection.0 <= self.max
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        IntegerSelection(0)
    }
}


impl ControllerInput{
    pub fn get_integer_selection_if_id(&self, id: ControllerID)->Option<IntegerSelection>{
        if id != self.id() {return None};
        let ControllerSelection::Integer(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_integer_selection<'a>(&self, game: &'a Game)->Option<&'a IntegerSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::Integer(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}

impl From<i8> for IntegerSelection {
    fn from(value: i8) -> Self {
        IntegerSelection(value)
    }
}