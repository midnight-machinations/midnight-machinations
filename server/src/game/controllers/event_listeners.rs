use crate::{
    game::prelude::*, 
    packet::ToClientPacket, vec_set::VecSet
};
impl Controllers{
    pub fn on_controller_input_received(
        game: &mut Game,
        event: &OnControllerInputReceived,
        _fold: &mut (),
        _priority: ()
    ){
        let ControllerInput{
            id, selection: incoming_selection
        } = event.input.clone();
        let actor = event.actor_ref;

        if !Self::set_selection_in_controller(game, Some(actor), id.clone(), incoming_selection.clone(), false) {
            return
        }

        if id.should_send_selection_chat_message(game) {
            Self::send_selection_message(game, actor, id, incoming_selection);
        }

        OnValidatedControllerInputReceived::new(actor, event.input.clone()).as_invokable().invoke(game);
    }

    pub fn on_tick(game: &mut Game, _event: &OnTick, _fold: &mut (), _priority: ()){
        Self::update_controllers_from_parameters(game);
    }

    pub fn send_controller_to_client(game: &mut Game, event: &OnControllerChanged, _fold: &mut (), _priority: ()){
        let mut players_to_remove = VecSet::new();
        let mut players_to_update = VecSet::new();

        if let Some(controller) = &event.old{
            players_to_remove.extend(controller.parameters.allowed_players().iter());
        }
        if let Some(controller) = &event.new{
            players_to_update.extend(controller.parameters.allowed_players().iter());
        }

        for player in players_to_remove.sub(&players_to_update) {
            player.send_packet(game, ToClientPacket::YourAllowedController {
                id: event.id.clone(),
                controller: None
            });
        }
        for player in players_to_update {
            player.send_packet(game, ToClientPacket::YourAllowedController {
                id: event.id.clone(),
                controller: event.new.clone()
            });
        }
    }
}
impl EventListener<OnPhaseStart> for ControllerID {
    fn on_event(self, game: &mut Game, data: &OnPhaseStart, _fold: &mut <OnPhaseStart as crate::game::event::EventData>::FoldValue, _priority: <OnPhaseStart as crate::game::event::EventData>::Priority) {
        let Some(controller) = game.controllers.controllers.get_mut(&self) else {return};
        let old = controller.clone();
        controller.reset_on_phase_start(data.phase.phase());

        if old != *controller {
            OnControllerChanged::new(
                self.clone(),
                Some(old),
                Some(controller.clone())
            ).as_invokable().invoke(game);
        }
    }
}
pub struct ControllersEventListenerHandle;
impl EventListener<OnPhaseStart> for ControllersEventListenerHandle {
    fn on_event(self, game: &mut Game, data: &OnPhaseStart, fold: &mut <OnPhaseStart as crate::game::event::EventData>::FoldValue, priority: <OnPhaseStart as crate::game::event::EventData>::Priority) {
        Controllers::update_controllers_from_parameters(game);
        game.controllers.all_controller_ids().into_iter().on_event(game, data, fold, priority);
    }
}