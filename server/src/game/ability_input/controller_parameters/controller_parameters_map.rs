use super::builder::ControllerParametersBuilder;

use crate::game::Game;
use crate::vec_map::{vec_map, VecMap};

use super::super::controller_id::ControllerID;

use super::*;



#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ControllerParametersMap{
    controllers: VecMap<ControllerID, ControllerParameters>
}
impl ControllerParametersMap{
    pub fn new(abilities: VecMap<ControllerID, ControllerParameters>)->Self{
        Self{controllers: abilities}
    }
    pub fn new_controller(id: ControllerID, ability_data: ControllerParameters)->Self{
        Self{
            controllers: vec_map!((id, ability_data))
        }
    }
    pub fn builder(game: &Game) -> ControllerParametersBuilder {
        ControllerParametersBuilder::new(game)
    }
    pub fn insert_ability(&mut self, id: ControllerID, ability_data: ControllerParameters){
        self.controllers.insert(id, ability_data);
    }
    pub fn combine_overwrite(&mut self, other: Self){
        for (ability_id, ability_selection) in other.controllers {
            self.controllers.insert(ability_id, ability_selection);
        }
    }
    pub fn combine(maps: impl IntoIterator<Item=ControllerParametersMap>) -> Self {
        let mut curr = ControllerParametersMap::default();

        for map in maps {
            curr.combine_overwrite(map);
        }
        
        curr
    }
    pub fn controller_parameters(&self)->&VecMap<ControllerID, ControllerParameters>{
        &self.controllers
    }
}