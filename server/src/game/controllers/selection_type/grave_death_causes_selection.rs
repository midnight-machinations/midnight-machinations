use serde::{Deserialize, Serialize};

use crate::game::{
    Game, components::graves::grave::GraveDeathCause, controllers::{
        AvailableSelectionKind, ControllerID, ControllerInput, controller_selection::ControllerSelection
    }
};


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraveDeathCausesSelection(pub Vec<GraveDeathCause>);


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AvailableGraveDeathCausesSelection;
impl AvailableSelectionKind for AvailableGraveDeathCausesSelection{
    type Selection = GraveDeathCausesSelection;
    fn validate_selection(&self, _game: &Game, _selection: &GraveDeathCausesSelection)->bool{
        true
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        GraveDeathCausesSelection(Vec::new())
    }
}


impl ControllerInput{
    pub fn get_grave_death_causes_selection_if_id(&self, id: ControllerID)->Option<GraveDeathCausesSelection>{
        if id != self.id() {return None};
        let ControllerSelection::GraveDeathCauses(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_grave_death_causes_selection<'a>(&self, game: &'a Game)->Option<&'a GraveDeathCausesSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::GraveDeathCauses(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}

impl From<Vec<GraveDeathCause>> for GraveDeathCausesSelection {
    fn from(value: Vec<GraveDeathCause>) -> Self {
        GraveDeathCausesSelection(value)
    }
}