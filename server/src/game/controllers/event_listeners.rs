use crate::{
    game::{
        controllers::{ControllerID, ControllerInput, Controllers}, event::{
            on_controller_changed::OnControllerChanged, on_controller_input_received::OnControllerInputReceived, on_game_start::OnGameStart, on_phase_start::OnPhaseStart, on_tick::OnTick, on_validated_ability_input_received::OnValidatedControllerInputReceived, Event
        }, event_handlers::{EventHandlers, EventListener, EventListenerParameters}, Game
    }, 
    packet::ToClientPacket, vec_set::VecSet
};

impl Controllers{
    pub fn on_game_start(game: &mut Game, _event: &OnGameStart, _fold: &mut (), _priority: ()){
        #[derive(Clone)]
        struct L;
        impl EventListener<OnTick> for L {
            fn call(_state: &(), game: &mut Game, _param: &mut EventListenerParameters<OnTick>) {
                Controllers::update_controllers_from_parameters(game);
            }
        }
        EventHandlers::register(game, L);
    }
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

        OnValidatedControllerInputReceived::new(actor, event.input.clone()).invoke(game);
    }

    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        Self::update_controllers_from_parameters(game);
        
        for id in ControllerID::current_used_ids(game){
            let Some(controller) = game.controllers.controllers.get_mut(&id) else {continue};
            let old = controller.clone();
            controller.reset_on_phase_start(event.phase.phase());

            if old != *controller {
                OnControllerChanged::new(
                    id.clone(),
                    Some(old),
                    Some(controller.clone())
                ).invoke(game);
            }
        }
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