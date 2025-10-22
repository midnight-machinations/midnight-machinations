use crate::{
    game::{event::{on_game_ending::OnGameEnding, on_phase_start::OnPhaseStart}, prelude::{event_listeners::ControllersEventListenerHandle, EventListener}},
    packet::ToClientPacket
};

use super::{
    chat::{ChatGroup, ChatMessageVariant},
    components::synopsis::SynopsisTracker,
    phase::{PhaseState, PhaseStateMachine},
    Game, GameOverReason
};

//Event listerner functions for game defined here
impl Game{
    pub fn on_phase_start(&mut self, _event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        <ControllersEventListenerHandle as EventListener<OnPhaseStart>>::on_event(ControllersEventListenerHandle, self, _event, _fold, _priority);
        self.send_packet_to_all(ToClientPacket::Phase { 
            phase: self.current_phase().clone(),
            day_number: self.phase_machine.day_number,
        });
        self.send_packet_to_all(ToClientPacket::PhaseTimeLeft{ seconds_left: self.phase_machine.time_remaining.map(|o|o.as_secs().try_into().expect("Phase time should be below 18 hours")) });
    }
    pub fn on_game_ending(&mut self, event: &OnGameEnding, _fold: &mut (), _priority: ()){
        let synopsis = SynopsisTracker::get(self, event.conclusion);

        PhaseStateMachine::next_phase(self, Some(PhaseState::Recess));
        self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::GameOver { synopsis });
        self.send_packet_to_all(ToClientPacket::GameOver{ reason: GameOverReason::Draw });
        
        self.ticking = false;
    }
}

