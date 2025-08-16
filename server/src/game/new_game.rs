
use super::event::before_initial_role_creation::BeforeInitialRoleCreation;
use rand::seq::SliceRandom;

use crate::{
    client_connection::ClientConnection,
    game::{
        ability_input::SavedControllersMap,
        chat::ChatComponent,
        components::{
            confused::Confused, cult::Cult, detained::Detained,
            drunk_aura::DrunkAura, fragile_vest::FragileVestsComponent,
            graves::Graves, insider_group::{InsiderGroupID, InsiderGroups},
            mafia::Mafia, mafia_recruits::MafiaRecruits, pitchfork::Pitchfork,
            poison::Poison,
            puppeteer_marionette::PuppeteerMarionette, silenced::Silenced,
            syndicate_gun_item::SyndicateGunItem, synopsis::SynopsisTracker,
            tags::Tags, verdicts_today::VerdictsToday,
            win_condition::WinConditionComponent
        },
        event::on_game_start::OnGameStart, game_client::GameClient,
        modifiers::Modifiers, phase::PhaseStateMachine,
        player::{Player, PlayerIndex, PlayerInitializeParameters, PlayerReference},
        role_list::RoleAssignment, role_outline_reference::RoleOutlineReference,
        settings::Settings,
        spectator::{
            spectator_pointer::SpectatorPointer, Spectator,SpectatorInitializeParameters
        },
        Assignments, Game, RejectStartReason
    },
    packet::ToClientPacket, room::RoomClientID, vec_map::VecMap
};

impl Game{
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
                
                graves: Graves::default(),
                insider_groups: unsafe{InsiderGroups::new(num_players, &assignments)},
                detained: Detained::default(),
                confused: Confused::default(),
                drunk_aura: DrunkAura::default(),
                synopsis_tracker: SynopsisTracker::new(num_players),
                tags: Tags::default(),
                silenced: Silenced::default(),
                fragile_vests: unsafe{FragileVestsComponent::new(num_players)},
                win_condition: unsafe{WinConditionComponent::new(num_players, &assignments)},
                chat_messages: unsafe{ChatComponent::new(num_players)}
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
    pub fn assign_players_to_assignments(
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
}