#![allow(clippy::get_first, reason = "Often need to get first two visits manually.")]

pub mod game_client;
pub mod grave;
pub mod phase;
pub mod player;
pub mod chat;
pub mod role;
pub mod visit;
pub mod verdict;
pub mod role_list;
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
pub mod ability_input;

use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;
use ability_input::saved_controllers_map::SavedControllersMap;
use ability_input::ControllerID;
use ability_input::PlayerListSelection;
use components::confused::Confused;
use components::drunk_aura::DrunkAura;
use components::enfranchise::Enfranchise;
use components::forfeit_vote::ForfeitVote;
use components::fragile_vest::FragileVests;
use components::mafia::Mafia;
use components::pitchfork::Pitchfork;
use components::mafia_recruits::MafiaRecruits;
use components::player_component::PlayerComponent;
use components::poison::Poison;
use components::detained::Detained;
use components::insider_group::InsiderGroupID;
use components::insider_group::InsiderGroups;
use components::silenced::Silenced;
use components::syndicate_gun_item::SyndicateGunItem;
use components::synopsis::SynopsisTracker;
use components::tags::Tags;
use components::verdicts_today::VerdictsToday;
use components::win_condition::WinCondition;
use event::on_tick::OnTick;
use modifiers::ModifierType;
use modifiers::Modifiers;
use event::before_initial_role_creation::BeforeInitialRoleCreation;
use rand::seq::SliceRandom;
use role_list::RoleAssignment;
use role_outline_reference::RoleOutlineReference;
use serde::Serialize;

use crate::client_connection::ClientConnection;
use crate::game::event::on_game_start::OnGameStart;
use crate::game::player::PlayerIndex;
use game_client::GameClient;
use game_client::GameClientLocation;
use crate::room::RoomClientID;
use crate::room::name_validation;
use crate::room::JoinRoomClientResult;
use crate::room::RemoveRoomClientResult;
use crate::room::RoomState;
use crate::room::RoomTickResult;
use crate::packet::HostDataPacketGameClient;
use crate::packet::RoomPreviewData;
use crate::packet::RejectJoinReason;
use crate::packet::ToClientPacket;
use crate::vec_map::VecMap;
use crate::vec_set::VecSet;
use crate::websocket_connections::connection::ClientSender;
use chat::{ChatMessageVariant, ChatGroup, ChatMessage};
use player::PlayerReference;
use player::Player;
use phase::PhaseStateMachine;
use settings::Settings;
use grave::Grave;
use self::components::{
    cult::Cult,
    puppeteer_marionette::PuppeteerMarionette
};
use self::game_conclusion::GameConclusion;
use self::event::on_game_ending::OnGameEnding;
use self::event::on_grave_added::OnGraveAdded;
use self::grave::GraveReference;
use self::phase::PhaseState;
use self::player::PlayerInitializeParameters;
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
    pub assignments: VecMap<PlayerReference, (RoleOutlineReference, RoleAssignment)>,

    pub players: Box<[Player]>,
    pub graves: Vec<Grave>,

    phase_machine : PhaseStateMachine,

    
    /// Whether the game is still updating phase times
    pub ticking: bool,
    
    
    //components with data
    pub saved_controllers: SavedControllersMap,
    syndicate_gun_item: SyndicateGunItem,
    pub cult: Cult,
    pub mafia: Mafia,
    pub puppeteer_marionette: PuppeteerMarionette,
    pub mafia_recruits: MafiaRecruits,
    pub verdicts_today: VerdictsToday,
    pub pitchfork: Pitchfork,
    pub poison: Poison,
    pub modifiers: Modifiers,
    pub insider_groups: InsiderGroups,
    pub detained: Detained,
    pub confused: Confused,
    pub drunk_aura: DrunkAura,
    pub synopsis_tracker: SynopsisTracker,
    pub tags: Tags,
    pub silenced: Silenced,
    pub fragile_vests: PlayerComponent<FragileVests>,
    pub win_condition: PlayerComponent<WinCondition>
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

type Assignments = VecMap<PlayerReference, (RoleOutlineReference, RoleAssignment)>;

impl Game {
    pub const DISCONNECT_TIMER_SECS: u16 = 60 * 2;

    /// `players` must have length 255 or lower.
    pub fn new(
        room_name: String,
        settings: Settings,
        clients: VecMap<RoomClientID, GameClient>,
        players: Vec<PlayerInitializeParameters>,
        spectators: Vec<SpectatorInitializeParameters>
    ) -> Result<Self, RejectStartReason>{
        //check settings are not completly off the rails
        if settings.phase_times.game_ends_instantly() {
            return Err(RejectStartReason::ZeroTimeGame);
        }
        

        let mut role_generation_tries = 0u8;
        const MAX_ROLE_GENERATION_TRIES: u8 = 250;
        let mut game = loop {

            if role_generation_tries >= MAX_ROLE_GENERATION_TRIES {
                return Err(RejectStartReason::RoleListCannotCreateRoles);
            }

            let settings = settings.clone();
            let role_list = settings.role_list.clone();

            let random_outline_assignments = match role_list.create_random_role_assignments(&settings.enabled_roles){
                Some(roles) => {roles},
                None => {
                    role_generation_tries = role_generation_tries.saturating_add(1);
                    continue;
                }
            };

            let assignments = Self::assign_players_to_assignments(random_outline_assignments);            


            // Create list of players
            let mut new_players = Vec::new();
            for (player_index, player) in players.iter().enumerate() {
                let Ok(player_index) = player_index.try_into() else {return Err(RejectStartReason::TooManyClients)};
                let player_ref = unsafe{PlayerReference::new_unchecked(player_index)};

                let ClientConnection::Connected(ref sender) = player.connection else {
                    return Err(RejectStartReason::PlayerDisconnected)
                };
                let Some((_, assignment)) = assignments.get(&player_ref) else {
                    return Err(RejectStartReason::RoleListTooSmall)
                };

                let new_player = Player::new(
                    player.name.clone(),
                    sender.clone(),
                    assignment.role()
                );
                
                new_players.push(new_player);
            }

            #[expect(clippy::cast_possible_truncation, reason = "Explained in doc comment")]
            let num_players = new_players.len() as u8;

            let mut game = Self{
                room_name: room_name.clone(),
                clients: clients.clone(),
                pitchfork: Pitchfork::new(num_players),

                assignments: assignments.clone(),
                ticking: true,
                spectators: spectators.clone().into_iter().map(Spectator::new).collect(),
                spectator_chat_messages: Vec::new(),
                players: new_players.into_boxed_slice(),
                graves: Vec::new(),
                phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
                modifiers: Modifiers::default_from_settings(settings.enabled_modifiers.clone()),
                settings,

                saved_controllers: SavedControllersMap::default(),
                syndicate_gun_item: SyndicateGunItem::default(),
                cult: Cult::default(),
                mafia: Mafia,
                puppeteer_marionette: PuppeteerMarionette::default(),
                mafia_recruits: MafiaRecruits::default(),
                verdicts_today: VerdictsToday::default(),
                poison: Poison::default(),

                insider_groups: unsafe{InsiderGroups::new(num_players, &assignments)},
                detained: Detained::default(),
                confused: Confused::default(),
                drunk_aura: DrunkAura::default(),
                synopsis_tracker: SynopsisTracker::new(num_players),
                tags: Tags::default(),
                silenced: Silenced::default(),
                fragile_vests: unsafe{PlayerComponent::<FragileVests>::new(num_players)},
                win_condition: unsafe{PlayerComponent::<WinCondition>::new(num_players, &assignments)}
            };

            // Just distribute insider groups, this is for game over checking (Keeps game running syndicate gun)
            for player in PlayerReference::all_players(&game){
                let Some((_, assignment)) = assignments.get(&player) else {
                    return Err(RejectStartReason::RoleListTooSmall)
                };
                
                let insider_groups = assignment.insider_groups();
                
                for group in insider_groups{
                    unsafe {
                        group.add_player_to_revealed_group_unchecked(&mut game, player);
                    }
                }
            }


            if !game.game_is_over() {
                break game;
            }
            role_generation_tries = role_generation_tries.saturating_add(1);
        };

        if game.game_is_over() {
            return Err(RejectStartReason::RoleListCannotCreateRoles);
        }
        
        game.send_packet_to_all(ToClientPacket::StartGame);

        //set wincons
        for player in PlayerReference::all_players(&game){
            // We already set this earlier, now we just need to call the on_convert event. Hope this doesn't end the game!
            let win_condition = player.win_condition(&game).clone();
            player.set_win_condition(&mut game, win_condition);
            InsiderGroups::send_player_insider_groups_packet(&game, player);
        }

        BeforeInitialRoleCreation::invoke(&mut game);
        
        //on role creation needs to be called after all players roles are known
        //trigger role event listeners
        for player_ref in PlayerReference::all_players(&game){
            player_ref.initial_role_creation(&mut game);
        }

        for player_ref in PlayerReference::all_players(&game){
            player_ref.send_join_game_data(&mut game);
        }
        for spectator in SpectatorPointer::all_spectators(&game){
            spectator.send_join_game_data(&mut game);
        }

        //reveal groups
        for group in InsiderGroupID::all() {
            group.reveal_group_players(&mut game);
        }

        //on game start needs to be called after all players have joined
        OnGameStart::invoke(&mut game);

        Ok(game)
    }
    
    /// `initialization_data` must have length 255 or lower
    #[expect(clippy::cast_possible_truncation, reason = "See doc comment")]
    fn assign_players_to_assignments(
        initialization_data: Vec<RoleAssignment>
    )->Assignments{

        let mut player_indices: Vec<PlayerIndex> = (0..initialization_data.len() as PlayerIndex).collect();
        //remove all players that are already assigned
        player_indices.retain(|p|!initialization_data.iter().any(|a|a.player() == Some(*p)));
        player_indices.shuffle(&mut rand::rng());

        initialization_data
            .into_iter()
            .enumerate()
            .map(|(o_index, assignment)|{

                let p_index = if let Some(player) = assignment.player() {
                    player
                }else{
                    player_indices.swap_remove(0)
                };

                // We are iterating through playerlist and outline list, so this unsafe should be fine
                unsafe {
                    (PlayerReference::new_unchecked(p_index), (RoleOutlineReference::new_unchecked(o_index as u8), assignment))
                }
            })
            .collect()
    }

    #[expect(clippy::cast_possible_truncation, reason = "Game can only have 255 players maximum")]
    pub fn num_players(&self) -> u8 {
        self.players.len() as u8
    }

    /// Returns a tuple containing the number of guilty votes and the number of innocent votes
    pub fn count_verdict_votes(&self, player_on_trial: PlayerReference)->(u8,u8){
        let mut guilty = 0u8;
        let mut innocent = 0u8;
        for player_ref in PlayerReference::all_players(self){
            if !player_ref.alive(self) || player_ref == player_on_trial {
                continue;
            }
            let mut voting_power = 1u8;
            if Enfranchise::enfranchised(self, player_ref) {
                voting_power = voting_power.saturating_add(2);
            }
            
            match player_ref.verdict(self) {
                Verdict::Innocent => innocent = innocent.saturating_add(voting_power),
                Verdict::Abstain => {},
                Verdict::Guilty => guilty = guilty.saturating_add(voting_power),
            }
        }
        (guilty, innocent)
    }
    
    /// this is sent to the players whenever this function is called
    fn create_voted_player_map(&self) -> VecMap<PlayerReference, u8> {
        let mut voted_player_votes: VecMap<PlayerReference, u8> = VecMap::new();

        for player in PlayerReference::all_players(self){
            if !player.alive(self) { continue }

            let Some(PlayerListSelection(voted_players)) = ControllerID::Nominate { player }.get_player_list_selection(self) else {continue};
            let Some(&voted_player) = voted_players.first() else { continue };
            

            let mut voting_power: u8 = 1;
            if Enfranchise::enfranchised(self, player) {
                voting_power = voting_power.saturating_add(2);
            }

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

        let voted_player_votes = self.create_voted_player_map();
        self.send_packet_to_all(ToClientPacket::PlayerVotes { votes_for_player: voted_player_votes.clone()});

        let mut voted_player = None;

        if let Some(maximum_votes) = voted_player_votes.values().max() {
            if self.nomination_votes_is_enough(*maximum_votes){
                let max_votes_players: VecSet<PlayerReference> = voted_player_votes.iter()
                    .filter(|(_, votes)| **votes == *maximum_votes)
                    .map(|(player, _)| *player)
                    .collect();

                if max_votes_players.count() == 1 {
                    voted_player = max_votes_players.iter().next().copied();
                }
            }
        }
        
        if start_trial_instantly {
            if let Some(player_on_trial) = voted_player {
                PhaseStateMachine::next_phase(self, Some(PhaseState::Testimony {
                    trials_left: trials_left.saturating_sub(1), 
                    player_on_trial, 
                    nomination_time_remaining: self.phase_machine.get_time_remaining()
                }));
            }
        }

        voted_player
    }

    
    pub fn nomination_votes_is_enough(&self, votes: u8)->bool{
        votes >= self.nomination_votes_required()
    }
    pub fn nomination_votes_required(&self)->u8{
        #[expect(clippy::cast_possible_truncation, reason = "Game can only have max 255 players")]
        let eligible_voters = PlayerReference::all_players(self)
            .filter(|p| p.alive(self) && !ForfeitVote::forfeited_vote(self, *p))
            .count() as u8;

        if Modifiers::is_enabled(self, ModifierType::TwoThirdsMajority) {
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


    pub fn game_is_over(&self) -> bool {
        GameConclusion::game_is_over(self).is_some()
    }

    pub fn current_phase(&self) -> &PhaseState {
        &self.phase_machine.current_state
    }

    pub fn day_number(&self) -> u8 {
        self.phase_machine.day_number
    }

    pub fn add_grave(&mut self, grave: Grave) {
        if let Ok(grave_index) = self.graves.len().try_into() {
            self.graves.push(grave.clone());

            if let Some(grave_ref) = GraveReference::new(self, grave_index) {
                OnGraveAdded::new(grave_ref).invoke(self);
            }
        }
    }

    pub fn add_message_to_chat_group(&mut self, group: ChatGroup, variant: ChatMessageVariant){
        let message = ChatMessage::new_non_private(variant.clone(), group.clone());

        for player_ref in group.all_players_in_group(self){
            player_ref.add_chat_message(self, message.clone());
            player_ref.send_chat_messages(self);
        }

        if group == ChatGroup::All {
            self.add_chat_message_to_spectators(variant);
        }
    }
    pub fn add_messages_to_chat_group(&mut self, group: ChatGroup, messages: Vec<ChatMessageVariant>){
        for message in messages {
            self.add_message_to_chat_group(group.clone(), message);
        }
    }
    pub fn add_chat_message_to_spectators(&mut self, message: ChatMessageVariant){
        for spectator in self.spectators.iter_mut(){
            spectator.queued_chat_messages.push(message.clone());
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
                .filter(|(&id, _)| skip.is_none_or(|s| s != id))
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

impl RoomState for Game {
    fn tick(&mut self, time_passed: Duration) -> RoomTickResult {
        if !self.ticking { 
            return RoomTickResult { close_room: false }
        }

        if let Some(conclusion) = GameConclusion::game_is_over(self) {
            OnGameEnding::new(conclusion).invoke(self);
        }

        if self.phase_machine.day_number == u8::MAX {
            self.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::GameOver { 
                synopsis: SynopsisTracker::get(self, GameConclusion::Draw)
            });
            self.send_packet_to_all(ToClientPacket::GameOver{ reason: GameOverReason::ReachedMaxDay });
            self.ticking = false;
            return RoomTickResult { close_room: !self.is_any_client_connected() };
        }

        while self.phase_machine.time_remaining.is_some_and(|d| d.is_zero()) {
            PhaseStateMachine::next_phase(self, None);
        }
        PlayerReference::all_players(self).for_each(|p|p.tick(self, time_passed));
        SpectatorPointer::all_spectators(self).for_each(|s|s.tick(self, time_passed));

        self.phase_machine.time_remaining = self.phase_machine.time_remaining.map(|d|d.saturating_sub(time_passed));

        OnTick::new().invoke(self);

        RoomTickResult {
            close_room: !self.is_any_client_connected()
        }
    }
    
    fn send_to_client_by_id(&self, room_client_id: RoomClientID, packet: ToClientPacket) {
        if let Some(player) = self.clients.get(&room_client_id) {
            match player.client_location {
                GameClientLocation::Player(player) => player.send_packet(self, packet),
                GameClientLocation::Spectator(spectator) => spectator.send_packet(self, packet)
            }
        }
    }
    
    fn join_client(&mut self, send: &ClientSender) -> Result<JoinRoomClientResult, RejectJoinReason> {
        let is_host = !self.clients.iter().any(|p|p.1.host);
                
        let Some(room_client_id) = 
            (self.clients
                .iter()
                .map(|(i,_)|*i)
                .fold(0u32, u32::max) as RoomClientID).checked_add(1) else {
                    return Err(RejectJoinReason::RoomFull);
                };

        self.ensure_host_exists(None);

        let new_spectator = self.join_spectator(SpectatorInitializeParameters {
            connection: ClientConnection::Connected(send.clone()),
            host: is_host,
        })?;
        
        let new_client = GameClient::new_spectator(new_spectator, is_host);

        self.clients.insert(room_client_id, new_client);

        self.resend_host_data_to_all_hosts();
        Ok(JoinRoomClientResult { id: room_client_id, in_game: true, spectator: true })
    }

    fn initialize_client(&mut self, room_client_id: RoomClientID, send: &ClientSender) {
        if let Some(client) = self.clients.get(&room_client_id) {
            match client.client_location {
                GameClientLocation::Player(player) => {
                    player.connect(self, send.clone());
                    player.send_join_game_data(self);
                },
                GameClientLocation::Spectator(spectator) => {
                    spectator.send_join_game_data(self);
                }
            }
        }
        
        send.send(ToClientPacket::PlayersHost{hosts:
            self.clients
                .iter()
                .filter(|p|p.1.host)
                .map(|p|*p.0)
                .collect()
        });

        send.send(ToClientPacket::RoomName { name: self.room_name.clone() });
    }
    
    fn remove_client(&mut self, room_client_id: u32) -> RemoveRoomClientResult {
        let Some(game_player) = self.clients.get_mut(&room_client_id) else {
            return RemoveRoomClientResult::ClientNotInRoom;
        };

        match game_player.client_location {
            GameClientLocation::Player(player) => player.quit(self),
            GameClientLocation::Spectator(spectator) => {
                self.clients.remove(&room_client_id);

                // Shift every other spectator down one index
                for client in self.clients.iter_mut() {
                    if let GameClientLocation::Spectator(ref mut other) = &mut client.1.client_location {
                        if other.index() > spectator.index() {
                            *other = SpectatorPointer::new(other.index().saturating_sub(1));
                        }
                    }
                }

                self.remove_spectator(spectator.index());
            }
        }

        self.ensure_host_exists(None);

        self.resend_host_data_to_all_hosts();

        if !self.is_any_client_connected() {
            RemoveRoomClientResult::RoomShouldClose
        } else {
            RemoveRoomClientResult::Success
        }
    }
    
    fn remove_client_rejoinable(&mut self, id: u32) -> RemoveRoomClientResult {
        let Some(game_player) = self.clients.get_mut(&id) else { return RemoveRoomClientResult::ClientNotInRoom };

        match game_player.client_location {
            GameClientLocation::Player(player) => {
                if !player.is_disconnected(self) {
                    player.lose_connection(self);
    
                    self.ensure_host_exists(None);
                    self.resend_host_data_to_all_hosts();
                }
            },
            GameClientLocation::Spectator(spectator) => {
                self.clients.remove(&id);

                // Shift every other spectator down one index
                for client in self.clients.iter_mut() {
                    if let GameClientLocation::Spectator(ref mut other) = &mut client.1.client_location {
                        if other.index() > spectator.index() {
                            *other = SpectatorPointer::new(other.index().saturating_sub(1));
                        }
                    }
                }

                self.remove_spectator(spectator.index());
            }
        }

        RemoveRoomClientResult::Success
    }
    
    fn rejoin_client(&mut self, _: &ClientSender, room_client_id: u32) -> Result<JoinRoomClientResult, RejectJoinReason> {
        let Some(client) = self.clients.get_mut(&room_client_id) else {
            return Err(RejectJoinReason::PlayerDoesntExist)
        };
        
        if let GameClientLocation::Player(player) = client.client_location {
            if !player.could_reconnect(self) {
                return Err(RejectJoinReason::PlayerTaken)
            };

            self.resend_host_data_to_all_hosts();

            Ok(JoinRoomClientResult { id: room_client_id, in_game: true, spectator: false })
        }else{
            Err(RejectJoinReason::PlayerDoesntExist)
        }
    }
    
    fn get_preview_data(&self) -> RoomPreviewData {
        RoomPreviewData {
            name: self.room_name.clone(),
            in_game: true,
            players: self.clients.iter()
                .filter_map(|(id, player)|
                    if let GameClientLocation::Player(player) = player.client_location {
                        Some((*id, player.name(self).clone()))
                    } else {
                        None
                    }
                )
                .collect()
        }
    }
    
    fn is_host(&self, room_client_id: u32) -> bool {
        if let Some(client) = self.clients.get(&room_client_id){
            client.host
        }else{
            false
        }
    }
}


pub mod test {

    use crate::vec_map::VecMap;

    use super::{
        ability_input::saved_controllers_map::SavedControllersMap, components::{
            cult::Cult, fragile_vest::FragileVests, insider_group::InsiderGroups,
            mafia::Mafia, mafia_recruits::MafiaRecruits, pitchfork::Pitchfork, player_component::PlayerComponent,
            poison::Poison, puppeteer_marionette::PuppeteerMarionette, silenced::Silenced, syndicate_gun_item::SyndicateGunItem,
            synopsis::SynopsisTracker, tags::Tags, verdicts_today::VerdictsToday, win_condition::WinCondition
        }, event::{before_initial_role_creation::BeforeInitialRoleCreation, on_game_start::OnGameStart},
        phase::PhaseStateMachine, player::{test::mock_player, PlayerReference},
        settings::Settings, Assignments, Game, RejectStartReason
    };
    
    pub fn mock_game(settings: Settings, num_players: u8) -> Result<(Game, Assignments), RejectStartReason> {

        //check settings are not completly off the rails
        if settings.phase_times.game_ends_instantly() {
            return Err(RejectStartReason::ZeroTimeGame);
        }

        let settings = settings.clone();
        let role_list = settings.role_list.clone();
        
        let random_outline_assignments = match role_list.create_random_role_assignments(&settings.enabled_roles){
            Some(roles) => {roles},
            None => {return Err(RejectStartReason::RoleListCannotCreateRoles);}
        };

        let assignments = Game::assign_players_to_assignments(random_outline_assignments);

        let mut players = Vec::new();
        for player in unsafe{PlayerReference::all_players_from_count(num_players)} {
            let new_player = mock_player(
                format!("{}",player.index()),
                match assignments.get(&player).map(|a|a.1.role()){
                    Some(role) => role,
                    None => return Err(RejectStartReason::RoleListTooSmall),
                }
            );
            players.push(new_player);
        }

        let mut game = Game{
            clients: VecMap::new(),
            room_name: "Test".to_string(),
            pitchfork: Pitchfork::new(num_players),
            
            assignments: assignments.clone(),
            ticking: true,
            spectators: Vec::new(),
            spectator_chat_messages: Vec::new(),
            players: players.into_boxed_slice(),
            graves: Vec::new(),
            phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
            settings,

            saved_controllers: SavedControllersMap::default(),
            syndicate_gun_item: SyndicateGunItem::default(),
            cult: Cult::default(),
            mafia: Mafia,
            puppeteer_marionette: PuppeteerMarionette::default(),
            mafia_recruits: MafiaRecruits::default(),
            verdicts_today: VerdictsToday::default(),
            poison: Poison::default(),
            modifiers: Default::default(),
            insider_groups: unsafe{InsiderGroups::new(num_players, &assignments)},
            detained: Default::default(),
            confused: Default::default(),
            drunk_aura: Default::default(),
            synopsis_tracker: SynopsisTracker::new(num_players),
            tags: Tags::default(),
            silenced: Silenced::default(),
            fragile_vests: unsafe{PlayerComponent::<FragileVests>::new(num_players)},
            win_condition: unsafe{PlayerComponent::<WinCondition>::new(num_players, &assignments)}
        };


        //set wincons
        for player in PlayerReference::all_players(&game){
            let role_data = player.role(&game).new_state(&game);
            player.set_win_condition(&mut game, role_data.clone().default_win_condition());
            InsiderGroups::send_player_insider_groups_packet(&game, player);
        }
        
        BeforeInitialRoleCreation::invoke(&mut game);

        //on role creation needs to be called after all players roles are known
        for player_ref in PlayerReference::all_players(&game){
            player_ref.initial_role_creation(&mut game);
        }

        OnGameStart::invoke(&mut game);

        Ok((game, assignments))
    }
}