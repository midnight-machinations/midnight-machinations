#![allow(clippy::get_first, reason = "Often need to get first two visits manually.")]

pub mod game_client;
pub mod phase;
pub mod player;
pub mod chat;
pub mod role;
pub mod verdict;
pub mod role_list;
pub mod role_list_generation;
pub mod settings;
pub mod game_conclusion;
pub mod components;
pub mod on_client_message;
pub mod event;
pub mod spectator;
pub mod game_listeners;
pub mod attack_power;
pub mod modifiers;
pub mod role_outline_reference;
pub mod controllers;
pub mod room_state;
pub mod new_game;
pub mod abilities_component;
pub mod abilities;
pub mod prelude;

use std::collections::VecDeque;
use std::time::Instant;
use crate::game::abilities_component::Abilities;
use crate::game::chat::PlayerChatGroups;
use crate::game::components::blocked::BlockedComponent;
use crate::game::components::fast_forward::FastForwardComponent;
use crate::game::components::pitchfork_item::PitchforkItemComponent;
use crate::game::components::role::RoleComponent;
use crate::game::components::role_reveal::RevealedPlayersComponent;
use crate::game::controllers::Controllers;
use crate::game::modifiers::ModifierID;
use controllers::ControllerID;
use controllers::PlayerListSelection;
use components::confused::Confused;
use components::enfranchise::EnfranchiseComponent;
use components::forfeit_vote::ForfeitNominationVote;
use components::mafia::Mafia;
use components::mafia_recruits::MafiaRecruits;
use components::poison::Poison;
use components::detained::Detained;
use components::insider_group::InsiderGroups;
use components::silenced::Silenced;
use components::synopsis::SynopsisTracker;
use components::tags::Tags;
use components::verdicts_today::VerdictsToday;
use rand::rngs::SmallRng;
use serde::Serialize;
use crate::client_connection::ClientConnection;
use crate::game::chat::ChatComponent;
use crate::game::components::fragile_vest::FragileVestsComponent;
use crate::game::components::graves::Graves;
use crate::game::components::win_condition::WinConditionComponent;
use crate::game::game_client::GameClient;
use crate::game::game_client::GameClientLocation;
use crate::game::modifiers::hidden_nomination_votes::HiddenNominationVotes;
use crate::game::role_list_generation::OutlineAssignment;
use crate::room::RoomClientID;
use crate::room::name_validation;
use crate::packet::HostDataPacketGameClient;
use crate::packet::RejectJoinReason;
use crate::packet::ToClientPacket;
use crate::vec_map::VecMap;
use crate::vec_set::VecSet;
use chat::{ChatMessageVariant, ChatGroup, ChatMessage};
use player::PlayerReference;
use player::Player;
use phase::PhaseStateMachine;
use settings::Settings;
use self::components::{
    cult::Cult,
    puppeteer_marionette::PuppeteerMarionette
};
use self::game_conclusion::GameConclusion;
use self::phase::PhaseState;
use self::spectator::{
    spectator_pointer::{
        SpectatorIndex, SpectatorPointer
    },
    Spectator,
    SpectatorInitializeParameters
};
use self::verdict::Verdict;


pub struct Game {
    room_name: String,
    clients: VecMap<RoomClientID, GameClient>,
    pub settings : Settings,

    pub spectators: Vec<Spectator>,
    pub spectator_chat_messages: Vec<ChatMessageVariant>,

    /// indexed by role outline reference
    pub assignments: Assignments,

    pub players: Box<[Player]>,

    phase_machine : PhaseStateMachine,

    
    /// Whether the game is still updating phase times
    pub ticking: bool,
    pub rng: SmallRng,
    
    
    //components with data
    pub player_chat_groups: PlayerChatGroups,
    pub revealed_players: RevealedPlayersComponent,
    pub graves: Graves,
    pub controllers: Controllers,
    pub abilities: Abilities,
    pub cult: Cult,
    pub mafia: Mafia,
    pub puppeteer_marionette: PuppeteerMarionette,
    pub mafia_recruits: MafiaRecruits,
    pub verdicts_today: VerdictsToday,
    pub blocked: BlockedComponent,
    pub pitchfork_item: PitchforkItemComponent,
    pub poison: Poison,
    pub insider_groups: InsiderGroups,
    pub detained: Detained,
    pub confused: Confused,
    pub synopsis_tracker: SynopsisTracker,
    pub tags: Tags,
    pub silenced: Silenced,
    pub enfranchise: EnfranchiseComponent,
    pub fragile_vests: FragileVestsComponent,
    pub win_condition: WinConditionComponent,
    pub role: RoleComponent,
    pub fast_forward: FastForwardComponent,
    pub chat_messages: ChatComponent
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum RejectStartReason {
    TooManyClients,
    GameEndsInstantly,
    RoleListTooSmall,
    RoleListCannotCreateRoles,
    ZeroTimeGame,
    PlayerDisconnected
}

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum GameOverReason {
    ReachedMaxDay,
    Winner,
    Draw
}

type Assignments = VecMap<PlayerReference, OutlineAssignment>;

impl Game {
    pub const DISCONNECT_TIMER_SECS: u16 = 60 * 2;

    

    #[expect(clippy::cast_possible_truncation, reason = "Game can only have 255 players maximum")]
    pub fn num_players(&self) -> u8 {
        self.players.len() as u8
    }

    /// Returns a tuple containing the number of guilty votes and the number of innocent votes
    pub fn count_verdict_votes(&self, player_on_trial: PlayerReference)->(u8,u8){
        let mut guilty = 0u8;
        let mut innocent = 0u8;
        for player in PlayerReference::all_players(self){
            if !player.alive(self) || player == player_on_trial {
                continue;
            }
            let voting_power = EnfranchiseComponent::voting_power(self, player);
            
            match player.verdict(self) {
                Verdict::Innocent => innocent = innocent.saturating_add(voting_power),
                Verdict::Abstain => {},
                Verdict::Guilty => guilty = guilty.saturating_add(voting_power),
            }
        }
        (guilty, innocent)
    }
    
    fn create_nominated_player_map(&self) -> VecMap<PlayerReference, u8> {
        let mut voted_player_votes: VecMap<PlayerReference, u8> = VecMap::new();

        for player in PlayerReference::all_players(self){
            if !player.alive(self) { continue }

            let Some(PlayerListSelection(voted_players)) = ControllerID::Nominate { player }.get_player_list_selection(self) else {continue};
            let Some(&voted_player) = voted_players.first() else { continue };
            

            let voting_power: u8 = EnfranchiseComponent::voting_power(self, player);

            if let Some(num_votes) = voted_player_votes.get_mut(&voted_player) {
                *num_votes = num_votes.saturating_add(voting_power);
            } else {
                voted_player_votes.insert(voted_player, voting_power);
            }
        }

        voted_player_votes
    }
    /// Returns the player who is meant to be put on trial
    /// None if its not nomination
    /// None if nobody has enough votes
    /// None if there is a tie
    pub fn count_nomination_and_start_trial(&mut self, start_trial_instantly: bool)->Option<PlayerReference>{

        let &PhaseState::Nomination { trials_left, .. } = self.current_phase() else {return None};

        let voted_player_votes = self.create_nominated_player_map();
        self.send_player_votes();

        let mut voted_player = None;

        if let Some(maximum_votes) = voted_player_votes.values().max() && self.nomination_votes_is_enough(*maximum_votes){
            let max_votes_players: VecSet<PlayerReference> = voted_player_votes.iter()
                .filter(|(_, votes)| **votes == *maximum_votes)
                .map(|(player, _)| *player)
                .collect();

            if max_votes_players.count() == 1 {
                voted_player = max_votes_players.iter().next().copied();
            }
        }
        
        if start_trial_instantly && let Some(player_on_trial) = voted_player {
            PhaseStateMachine::next_phase(self, Some(PhaseState::Testimony {
                trials_left: trials_left.saturating_sub(1), 
                player_on_trial, 
                nomination_time_remaining: self.phase_machine.get_time_remaining()
            }));
        }

        voted_player
    }

    
    pub fn nomination_votes_is_enough(&self, votes: u8)->bool{
        votes >= self.nomination_votes_required()
    }
    pub fn nomination_votes_required(&self)->u8{
        if self.modifier_settings().is_enabled(ModifierID::NoMajority) {return 0}

        #[expect(clippy::cast_possible_truncation, reason = "Game can only have max 255 players")]
        let eligible_voters = PlayerReference::all_players(self)
            .filter(|p| p.alive(self) && !ForfeitNominationVote::forfeited_vote(self, *p))
            .count() as u8;

        if self.modifier_settings().is_enabled(ModifierID::TwoThirdsMajority) {
            // equivalent to x - (x - (x + 1)/3)/2 to prevent overflow issues
            eligible_voters
            .saturating_sub(
                eligible_voters
                .saturating_sub(
                    eligible_voters
                    .saturating_add(1)
                    .saturating_div(3)
                )
                .saturating_div(2)
            )
        } else {
            eligible_voters.saturating_div(2).saturating_add(1)
        }
    }

    pub fn modifier_settings(&self) -> &modifiers::ModifierSettings {
        &self.settings.modifiers
    }

    pub fn game_is_over(&self) -> bool {
        GameConclusion::game_is_over_game(self).is_some()
    }

    pub fn current_phase(&self) -> &PhaseState {
        &self.phase_machine.current_state
    }

    pub fn day_number(&self) -> u8 {
        self.phase_machine.day_number
    }

    pub fn add_message_to_chat_group(&mut self, group: ChatGroup, variant: ChatMessageVariant){
        let message = ChatMessage::new_non_private(variant.clone(), group);

        for player_ref in group.all_players_in_group(self){
            player_ref.add_chat_message(self, message.clone());
            player_ref.send_chat_messages(self);
        }

        if group == ChatGroup::All {
            self.add_chat_message_to_spectators(variant);
        } else if group == ChatGroup::Spectator {
            self.add_chat_message_to_spectators(variant);
        }
    }
    pub fn add_messages_to_chat_group(&mut self, group: ChatGroup, messages: Vec<ChatMessageVariant>){
        for message in messages {
            self.add_message_to_chat_group(group, message);
        }
    }
    pub fn add_chat_message_to_spectators(&mut self, message: ChatMessageVariant){
        let new_idx = self.spectator_chat_messages.len();
        for spectator in self.spectators.iter_mut(){
            spectator.queued_chat_messages.push_back((new_idx, message.clone()));
        }
        self.spectator_chat_messages.push(message);
    }
    pub fn join_spectator(&mut self, params: SpectatorInitializeParameters) -> Result<SpectatorPointer, RejectJoinReason> {
        let spectator_index = SpectatorIndex::try_from(self.spectators.len()).map_err(|_|RejectJoinReason::RoomFull)?;
        self.spectators.push(Spectator::new(params));
        let spectator_pointer = SpectatorPointer::new(spectator_index);

        Ok(spectator_pointer)
    }
    pub fn remove_spectator(&mut self, i: SpectatorIndex){
        if (i as usize) < self.spectators.len() {
            self.spectators.remove(i as usize);
        }
    }

    pub fn send_packet_to_all(&self, packet: ToClientPacket){
        for player_ref in PlayerReference::all_players(self){
            player_ref.send_packet(self, packet.clone());
        }
        for spectator in self.spectators.iter(){
            spectator.send_packet(packet.clone());
        }
    }
    
    pub(crate) fn is_any_client_connected(&self) -> bool {
        PlayerReference::all_players(self).any(|p| p.is_connected(self))
        || SpectatorPointer::all_spectators(self).any(|s| s.is_connected(self))
    }

    fn ensure_host_exists(&mut self, skip: Option<RoomClientID>) {
        fn is_player_not_disconnected(game: &Game, p: &GameClient) -> bool {
            match p.client_location {
                GameClientLocation::Spectator(spectator) => {
                    !matches!(spectator.connection(game), ClientConnection::Disconnected)
                },
                GameClientLocation::Player(player) => {
                    !matches!(player.connection(game), ClientConnection::Disconnected)
                }
            }
        }
        fn is_player_not_disconnected_host(game: &Game, p: &GameClient) -> bool {
            p.host && is_player_not_disconnected(game, p)
        }

        if !self.clients.iter().any(|p| is_player_not_disconnected_host(self, p.1)) {
            let next_available_player_id = self.clients.iter()
                .filter(|(id, _)| skip.is_none_or(|s| s != **id))
                .filter(|(_, c)| is_player_not_disconnected(self, c))
                .map(|(&id, _)| id)
                .next();

            let next_available_player = next_available_player_id.map(|id| unsafe { self.clients.get_unchecked_mut(&id) });

            if let Some(new_host) = next_available_player {
                new_host.set_host();
            } else if let Some(new_host) = self.clients.values_mut().next(){
                new_host.set_host();
            }
        }
    }


    fn resend_host_data_to_all_hosts(&self) {
        for client in self.clients.values().filter(|client| client.host) {
            let client_connection = match client.client_location {
                GameClientLocation::Player(player) => player.connection(self).clone(),
                GameClientLocation::Spectator(spectator) => spectator.connection(self)
            };

            self.resend_host_data(&client_connection)
        }
    }
    
    fn resend_host_data(&self, connection: &ClientConnection) {
        connection.send_packet(ToClientPacket::HostData { clients: self.clients.iter()
            .map(|(id, client)| {
                (*id, HostDataPacketGameClient {
                    client_type: client.client_location.clone(),
                    connection: match client.client_location {
                        GameClientLocation::Player(player) => player.connection(self).clone(),
                        GameClientLocation::Spectator(spectator) => spectator.connection(self)
                    },
                    host: client.host
                })
            }).collect()
        });
    }

    fn send_players(&mut self){
        let players: Vec<String> = PlayerReference::all_players(self).map(|p|
            p.name(self).clone()
        ).collect();

        let packet = ToClientPacket::GamePlayers{ 
            players
        };

        self.send_packet_to_all(packet.clone());
    }

    fn send_player_votes(&mut self){
        self.send_packet_to_all(
            ToClientPacket::PlayerVotes{
                votes_for_player: 
                    if !HiddenNominationVotes::nomination_votes_are_hidden(self) {
                        self.create_nominated_player_map()
                    }else{
                        VecMap::new()
                    }
            }
        );
    }

    pub fn set_player_name(&mut self, player_ref: PlayerReference, name: String) {
        let mut other_players: Vec<String> = PlayerReference::all_players(self)
            .map(|p| p.name(self))
            .cloned()
            .collect();

        other_players.remove(player_ref.index() as usize);
        
        let new_name: String = name_validation::sanitize_name(name, &other_players);

        player_ref.set_name(self, new_name);
    }
    
    fn send_to_all(&self, packet: ToClientPacket) {
        self.send_packet_to_all(packet.clone())
    }
    
    pub fn get_client_last_message_times(&mut self, room_client_id: u32) -> Option<&mut VecDeque<Instant>> {
        if let Some(client) = self.clients.get_mut(&room_client_id) {
            Some(&mut client.last_message_times)
        } else {
            None
        }
    }
}

pub mod test;