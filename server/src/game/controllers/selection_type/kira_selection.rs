use serde::{Deserialize, Serialize};

use crate::{
    game::{
        controllers::{
            controller_selection::ControllerSelection, ControllerInput, ControllerID, AvailableSelectionKind
        },
        player::PlayerReference, role::kira::KiraGuess, Game
    },
    vec_map::VecMap,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct KiraSelection(pub VecMap<PlayerReference, KiraGuess>);
impl KiraSelection{
    pub fn new(map: VecMap<PlayerReference, KiraGuess>) -> Self{
        KiraSelection(map)
    }
}

impl PartialOrd for KiraSelection{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for KiraSelection{
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AvailableKiraSelection{
    pub count_must_guess: u8
}
impl AvailableSelectionKind for AvailableKiraSelection{
    type Selection = KiraSelection;
    fn validate_selection(&self, game: &Game, selection: &KiraSelection)->bool{
        let count: usize = self.count_must_guess.into();
        selection.0.len() == count &&
        selection.0.iter().all(|p|p.0.alive(game))
    }
    
    fn default_selection(&self, _: &Game) -> Self::Selection {
        KiraSelection(VecMap::new())
    }
}

impl AvailableKiraSelection{
    pub fn new(count_must_guess: u8)->Self{
        AvailableKiraSelection { count_must_guess }
    }
}


impl ControllerInput{
    pub fn get_kira_selection_if_id(&self, id: ControllerID)->Option<KiraSelection>{
        if id != self.id() {return None};
        let ControllerSelection::Kira(selection) = self.selection() else {return None};
        Some(selection)
    }
}
impl ControllerID{
    pub fn get_kira_selection<'a>(&self, game: &'a Game)->Option<&'a KiraSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::Kira(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
}