use crate::{game::{controllers::{Controller, ControllerID, ControllerParametersMap, Controllers}, player::PlayerReference}, vec_map::VecMap, vec_set::VecSet};

impl Controllers{
    // new query
    pub fn all_controllers(&self)->&VecMap<ControllerID, Controller>{
        &self.controllers
    }
    pub fn all_controller_ids(&self)->VecSet<ControllerID>{
        self.controllers.iter()
            .map(|(c, _)|c.clone())
            .collect()
    }

    pub fn controllers_allowed_to_player(&self, player: PlayerReference)->Controllers{
        Controllers::new(
            self.controllers.iter()
                .filter(|(_, saved_controller)| saved_controller.parameters.allowed_players().contains(&player))
                .map(|(id, saved_controller)| (id.clone(), saved_controller.clone()))
                .collect()
        )
    }
    
    pub fn controller_parameters(&self)->ControllerParametersMap{
        ControllerParametersMap::new(
            self.controllers.iter()
                .map(|(id, saved_controller)| (id.clone(), saved_controller.parameters.clone()))
                .collect()
        )
    }
    
    pub fn controller_parameters_allowed_to_player(&self, player: PlayerReference)->ControllerParametersMap{
        ControllerParametersMap::new(
            self.controller_parameters().controller_parameters().iter()
                .filter(|(_, saved_controller)| saved_controller.allowed_players().contains(&player))
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect()
        )
    }
}