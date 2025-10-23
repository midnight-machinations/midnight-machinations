use crate::game::event::propagate::EventListener;

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

    pub fn on_phase_start(game: &mut Game, data: &OnPhaseStart, fold: &mut (), priority: ()){
        Controllers::update_controllers_from_parameters(game);
        game.controllers.controllers
            .keys()
            .cloned()
            .collect::<Vec<ControllerID>>()
            .into_iter()
            .for_each(|id|id.on_event(game, data, fold, priority));
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
    fn on_event(&self, game: &mut Game, data: &OnPhaseStart, _fold: &mut <OnPhaseStart as crate::game::event::EventData>::FoldValue, _priority: <OnPhaseStart as crate::game::event::EventData>::Priority) {
        let Some(mut controller) = game.controllers.controllers.get(self).cloned() else {return};
        controller.reset_on_phase_start(data.phase.phase());
        Controllers::set_controller(game, self.clone(), Some(controller));
    }
}