use std::time::Duration;

use crate::{
    client_connection::ClientConnection, 
    game::{
        chat::{ChatComponent, ChatMessageVariant},
        components::{
            graves::grave_reference::GraveReference, insider_group::InsiderGroups,
            tags::Tags
        },
        Game, GameOverReason
    },
    packet::ToClientPacket, websocket_connections::connection::ClientSender
};

use super::PlayerReference;

impl PlayerReference{
    pub fn connect(&self, game: &mut Game, sender: ClientSender){
        self.deref_mut(game).connection = ClientConnection::Connected(sender);
        self.send_join_game_data(game);
    }
    pub fn lose_connection(&self, game: &mut Game){
        self.deref_mut(game).connection = ClientConnection::CouldReconnect { disconnect_timer: Duration::from_secs(Game::DISCONNECT_TIMER_SECS as u64) };
    }
    pub fn quit(&self, game: &mut Game) {
        self.deref_mut(game).connection = ClientConnection::Disconnected;
        if self.alive(game) {
            game.add_message_to_chat_group(
                crate::game::chat::ChatGroup::All, 
                ChatMessageVariant::PlayerQuit{player_index: self.index(), game_over: game.game_is_over()}
            );
        }
    }

    pub fn connection<'a>(&self, game: &'a Game) -> &'a ClientConnection {
        &self.deref(game).connection
    }
    pub fn is_connected(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::Connected(_))
    }
    pub fn could_reconnect(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::CouldReconnect {..})
    }
    pub fn is_disconnected(&self, game: &Game) -> bool {
        matches!(self.deref(game).connection, ClientConnection::Disconnected)
    }

    pub fn send_packet(&self, game: &Game, packet: ToClientPacket){
        self.deref(game).connection.send_packet(packet);
    }
    pub fn send_packets(&self, game: &Game, packets: Vec<ToClientPacket>){
        for packet in packets{
            self.send_packet(game, packet);
        }
    }
    pub fn send_repeating_data(&self, game: &mut Game){
        self.send_chat_messages(game);
    }
    pub fn send_join_game_data(&self, game: &mut Game){
        // General
        self.send_packets(game, vec![
            ToClientPacket::GamePlayers{ 
                players: PlayerReference::all_players(game).map(|p|p.name(game).clone()).collect()
            },
            ToClientPacket::EnabledRoles { roles: game.settings.enabled_roles.clone().into_iter().collect() },
            ToClientPacket::RoleList {role_list: game.settings.role_list.clone()},
            ToClientPacket::EnabledModifiers {
                modifiers: game.settings.enabled_modifiers.clone().into_iter().collect()
            },
            ToClientPacket::PlayerAlive{
                alive: PlayerReference::all_players(game).map(|p|p.alive(game)).collect()
            }
        ]);

        if !game.ticking {
            self.send_packet(game, ToClientPacket::GameOver { reason: GameOverReason::Draw })
        }


        game.send_player_votes();
        for grave in GraveReference::all_graves(game){
            self.send_packet(game, ToClientPacket::AddGrave {
                grave: grave.deref(game).clone(),
                grave_ref: grave
            });
        }

        // Player specific
        ChatComponent::requeue_chat_messages(game, *self);
        self.send_chat_messages(game);
        InsiderGroups::send_player_insider_groups_packet(game, *self);
        InsiderGroups::send_fellow_insiders_packets(game, *self);
        Tags::send_to_client(game, *self);

        self.send_packets(game, vec![
            ToClientPacket::YourSendChatGroups {
                send_chat_groups: self.get_current_send_chat_groups(game).into_iter().collect()
            },
            ToClientPacket::YourPlayerIndex { 
                player_index: self.index() 
            },
            ToClientPacket::YourRoleState {
                role_state: self.role_state(game).clone().get_client_role_state(game, *self)
            },
            ToClientPacket::YourRoleLabels { 
                role_labels: PlayerReference::ref_vec_map_to_index(self.revealed_players_map(game)) 
            },
            ToClientPacket::YourAllowedControllers { 
                save: game.saved_controllers.controllers_allowed_to_player(*self).all_controllers().clone(),
            },
            ToClientPacket::YourNotes{
                notes: self.notes(game).clone()
            },
            ToClientPacket::YourCrossedOutOutlines{
                crossed_out_outlines: self.crossed_out_outlines(game).clone()
            },
            ToClientPacket::Phase { 
                phase: game.current_phase().clone(),
                day_number: game.phase_machine.day_number 
            },
            ToClientPacket::PhaseTimeLeft { seconds_left: game.phase_machine.time_remaining.map(|o|o.as_secs().try_into().expect("Phase time should be below 18 hours")) },
            ToClientPacket::GameInitializationComplete
        ]);
    }



    pub fn send_chat_messages(&self, game: &mut Game){
        if ChatComponent::not_sent_messages(game, *self).is_empty() {
            return;
        }
        
        let mut chat_messages_out = vec![];

        // Send in chunks
        for _ in 0..5 {
            let msg_option = ChatComponent::not_sent_messages(game, *self).front();
            if let Some(msg) = msg_option{
                chat_messages_out.push(msg.clone());
                ChatComponent::not_sent_messages_pop_front(game, *self);
            } else {break}
        }
        
        self.send_packet(game, ToClientPacket::AddChatMessages {
            chat_messages: chat_messages_out.into_iter().collect()
        });
        

        self.send_chat_messages(game);
    }
}

