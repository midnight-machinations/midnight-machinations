pub mod selection_type; pub use selection_type::*;
pub mod controller_selection; pub use controller_selection::*;
pub mod controller; pub use controller::*;
pub mod controller_id; pub use controller_id::*;
pub mod controller_parameters; pub use controller_parameters::*;
pub mod controller_input; pub use controller_input::*;
pub mod event_listeners;
pub mod getters;

use serde::{Deserialize, Serialize};

use crate::{
    game::{
        abilities_component::Abilities, chat::{ChatComponent, ChatMessageVariant}, components::{
            alibi::Alibi, call_witness::CallWitness, forfeit_vote::ForfeitNominationVote, forward_messages::ForwardMessages,
            insider_group::InsiderGroupID, judgement_controller::JudgementController, mafia::Mafia,
            nomination_controller::NominationController
        }, event::{
            on_controller_changed::OnControllerChanged, AsInvokable as _, Invokable as _,
        }, player::PlayerReference, Game
    },
    vec_map::VecMap
};


#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Controllers{
    pub(super) controllers: VecMap<ControllerID, Controller>,
}

impl Controllers{
    pub fn new(controllers: VecMap<ControllerID, Controller>)->Self{
        Self{controllers}
    }


    // mutators
    fn update_controllers_from_parameters(game: &mut Game){
        let current_controller_parameters = &game.controllers.controller_parameters();

        let new_controller_parameters_map = ControllerParametersMap::combine([
            NominationController::controller_parameters_map(game),
            JudgementController::controller_parameters_map(game),
            Mafia::controller_parameters_map(game),
            ForfeitNominationVote::controller_parameters_map(game),
            Abilities::controller_parameters_map(game),
            ForwardMessages::controller_parameters_map(game),
            ChatComponent::controller_parameters_map(game),
            Alibi::controller_parameters_map(game),
            CallWitness::controller_parameters_map(game)
        ]);

        if *current_controller_parameters != new_controller_parameters_map {
            Self::set_controller_parameters(game, new_controller_parameters_map);
        }
    }

    pub fn send_selection_message(
        game: &mut Game,
        player_ref: PlayerReference,
        id: ControllerID,
        selection: ControllerSelection
    ){
        let chat_message = ChatMessageVariant::AbilityUsed{
            player: player_ref,
            ability_id: id,
            selection: selection.clone()
        };

        let mut target_message_sent = false;
        for insider_group in InsiderGroupID::all_groups_with_player(game, player_ref){
            game.add_message_to_chat_group( insider_group.get_insider_chat_group(), chat_message.clone());
            target_message_sent = true;
        }
        if !target_message_sent{
            player_ref.add_private_chat_message(game, chat_message);
        }
    }
    
    /// Keeps old selection if its valid, otherwise uses default_selection,
    /// even if default selection is invalid
    fn set_controller_parameters(game: &mut Game, new_controller_parameters_map: ControllerParametersMap){

        let controller_ids_to_remove = game.controllers.controller_parameters().controller_parameters().keys()
            .filter(|id| !new_controller_parameters_map.controller_parameters().contains_key(id))
            .cloned()
            .collect::<Vec<_>>();

        for id in controller_ids_to_remove{
            let old = game.controllers.controllers.remove(&id)
                .map(|(_, c)|c); 
            OnControllerChanged::new(id.clone(), old, None).as_invokable().invoke(game);
        }

        for (id, controller_parameters) in new_controller_parameters_map.controller_parameters().iter(){
            let mut new_selection = controller_parameters.default_selection().clone();
            
            if
                let Some(Controller{selection: old_selection, ..}) = game.controllers.controllers.get(id) &&
                controller_parameters.validate_selection(game, old_selection) &&
                !controller_parameters.dont_save() &&
                !controller_parameters.grayed_out()
            {
                new_selection = old_selection.clone();
            }

            let old = game.controllers.controllers.get(id).cloned();
            let new = Controller::new(
                    new_selection,
                    controller_parameters.clone()
                );
            game.controllers.controllers.insert(id.clone(), new.clone());

            let new = Some(new);
            if old != new {
                OnControllerChanged::new(id.clone(), old, new).as_invokable().invoke(game);
            }
        }
    }

    /// return true if selection was valid
    /// return false if selection was invalid (ie wrong actor)
    /// if selection is invalid then nothing happens, nothing is updated
    pub fn set_selection_in_controller(
        game: &mut Game,
        actor: Option<PlayerReference>,
        id: ControllerID,
        new_selection: impl Into<ControllerSelection>,
        overwrite_gray_out: bool
    )->bool{
        let new_selection = new_selection.into();
        
        Self::update_controllers_from_parameters(game);

        if !Self::validate_input(game, &id, &new_selection, overwrite_gray_out, actor){return false}

        let Some(controller) = game.controllers.controllers.get_mut(&id) else {return false};

        let old = Some(controller.clone());


        if !controller.parameters.dont_save() {
            controller.selection = new_selection;
            let new = Some(controller.clone());
            if old != new {
                OnControllerChanged::new(id, old, new).as_invokable().invoke(game);
            }
        }

        true
    }

    fn validate_input(game: &Game, id: &ControllerID, selection: &ControllerSelection, overwrite_gray_out: bool, actor: Option<PlayerReference>)->bool{
        let Some(Controller {
            selection: saved_selection,
            parameters
        }) = game.controllers.controllers.get(id) else {return false};
        
        // validate input using available selection

        parameters.validate_selection(game, selection) &&   //parameters say its valid
        (overwrite_gray_out || !parameters.grayed_out()) && //not grayed out
        actor.is_none_or(|p|parameters.allowed_players().contains(&p)) &&   //actor is allowed
        (*saved_selection != *selection || *selection == ControllerSelection::Unit(UnitSelection))  //Something is actually changing (i think this can be removed?)
    }
}