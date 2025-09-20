use crate::{
    game::event::{on_game_ending::OnGameEnding, on_grave_added::OnGraveAdded, on_phase_start::OnPhaseStart},
    packet::ToClientPacket
};

use super::{
    chat::{ChatGroup, ChatMessageVariant}, components::synopsis::SynopsisTracker,
    event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority},
    phase::{PhaseState, PhaseStateMachine, PhaseType},
    player::PlayerReference, Game, GameOverReason
};

//Event listerner functions for game defined here
impl Game{
    pub fn on_phase_start(&mut self, _event: &OnPhaseStart, _fold: &mut (), _priority: ()){
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
    pub fn on_grave_added(game: &mut Game, event: &OnGraveAdded, _fold: &mut (), _priority: ()){   
        let grave = event.grave.deref(game).clone();     
        game.send_packet_to_all(ToClientPacket::AddGrave{grave: grave.clone(), grave_ref: event.grave});
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PlayerDied { grave: grave.clone() });

        
        for other_player_ref in PlayerReference::all_players(game){
            other_player_ref.conceal_players_role(game, grave.player);
        }
    }
    pub fn on_whisper(&mut self, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        match priority {
            WhisperPriority::Cancel => {
                if 
                    self.current_phase().phase() == PhaseType::Night || 
                    !event.receiver.alive(self) ||
                    !event.sender.alive(self) ||
                    event.receiver == event.sender || 
                    !event.sender.get_current_send_chat_groups(self).contains(&ChatGroup::All) ||
                    event.message.replace(['\n', '\r'], "").trim().is_empty()
                {
                    fold.cancelled = true;
                    fold.hide_broadcast = true;
                }
            },
            WhisperPriority::Broadcast => {
                if !fold.hide_broadcast {
                    self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::BroadcastWhisper {
                        whisperer: event.sender,
                        whisperee: event.receiver
                    });
                }
            },
            WhisperPriority::Send => {
                if fold.cancelled {
                    event.sender.add_private_chat_message(self, ChatMessageVariant::InvalidWhisper);
                } else {
                    let message = ChatMessageVariant::Whisper { 
                        from_player_index: event.sender, 
                        to_player_index: event.receiver, 
                        text: event.message.clone()
                    };

                    event.sender.add_private_chat_message(self, message.clone());
                    event.receiver.add_private_chat_message(self, message);
                }
            },
        }
    }
}

