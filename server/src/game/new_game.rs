use crate::{
    client_connection::ClientConnection,
    game::{
        abilities_component::Abilities, chat::{ChatComponent, PlayerChatGroups},
        components::{
            confused::Confused, cult::Cult, detained::Detained, fast_forward::FastForwardComponent, fragile_vest::FragileVestsComponent, graves::Graves, insider_group::{InsiderGroupID, InsiderGroups}, mafia::Mafia, mafia_recruits::MafiaRecruits, pitchfork_item::PitchforkItemComponent, poison::Poison, puppeteer_marionette::PuppeteerMarionette, role::RoleComponent, role_reveal::RevealedPlayersComponent, silenced::Silenced, synopsis::SynopsisTracker, tags::Tags, verdicts_today::VerdictsToday, win_condition::WinConditionComponent
        },
        controllers::Controllers,
        event::{on_game_start::OnGameStart, Event}, game_client::GameClient, modifiers::ModifierID, phase::PhaseStateMachine,
        player::{Player, PlayerInitializeParameters, PlayerReference},
        role_list_generation::{OutlineListAssignment, RoleListGenerator}, settings::Settings,
        spectator::{spectator_pointer::SpectatorPointer, Spectator, SpectatorInitializeParameters}, Assignments,
        Game, RejectStartReason
    },
    packet::ToClientPacket, room::{name_validation::generate_random_name, RoomClientID},
    vec_map::VecMap
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

            let mut role_list_generator = RoleListGenerator::new(settings.clone());

            let outline_list_assignment = match role_list_generator.generate_role_list() {
                Some(assignment) => {assignment},
                None => {
                    role_generation_tries = role_generation_tries.saturating_add(1);
                    continue;
                }
            };

            let assignments = Self::create_assignments(outline_list_assignment);        
            if assignments.len() != players.len() {
                return Err(RejectStartReason::RoleListTooSmall)
            }    


            // Create list of players
            let mut new_players = Vec::new();
            let mut new_players_names = Vec::new();
            for player in players.iter() {
                let ClientConnection::Connected(ref sender) = player.connection else {
                    return Err(RejectStartReason::PlayerDisconnected)
                };
                
                let name = if settings.modifiers.is_enabled(ModifierID::RandomPlayerNames) {
                    generate_random_name(
                        &new_players_names
                            .iter()
                            .map(|p: &String|p.as_str())
                            .collect::<Vec<&str>>()
                    )
                }else{
                    player.name.clone()
                };
                new_players_names.push(name.clone());

                let new_player = Player::new(
                    name,
                    sender.clone()
                );
                
                new_players.push(new_player);
            }

            #[expect(clippy::cast_possible_truncation, reason = "Explained in doc comment")]
            let num_players = new_players.len() as u8;

            let mut game = Self{
                room_name: room_name.clone(),
                clients: clients.clone(),
                // pitchfork: Pitchfork::new(num_players),

                assignments: assignments.clone(),
                ticking: true,
                spectators: spectators.clone().into_iter().map(Spectator::new).collect(),
                spectator_chat_messages: Vec::new(),
                players: new_players.into_boxed_slice(),
                phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
                settings,

                player_chat_groups: PlayerChatGroups::new(),
                revealed_players: unsafe{RevealedPlayersComponent::new(num_players)},
                controllers: Controllers::default(),
                abilities: Abilities::new(&assignments),
                cult: Cult::default(),
                mafia: Mafia,
                puppeteer_marionette: PuppeteerMarionette::default(),
                mafia_recruits: MafiaRecruits::default(),
                verdicts_today: VerdictsToday::default(),
                poison: Poison::default(),
                pitchfork_item: unsafe{PitchforkItemComponent::new(num_players)},
                graves: Graves::default(),
                insider_groups: unsafe{InsiderGroups::new(num_players, &assignments)},
                detained: Detained::default(),
                confused: Confused::default(),
                synopsis_tracker: SynopsisTracker::new(num_players),
                tags: Tags::default(),
                silenced: Silenced::default(),
                fragile_vests: unsafe{FragileVestsComponent::new(num_players)},
                win_condition: unsafe{WinConditionComponent::new(num_players, &assignments)},
                role: unsafe{RoleComponent::new(num_players, &assignments)},
                fast_forward: unsafe{FastForwardComponent::new(num_players)},
                chat_messages: unsafe{ChatComponent::new(num_players)}
            };

            for player in PlayerReference::all_players(&game){
                let Some(assignment) = assignments.get(&player) else {
                    return Err(RejectStartReason::RoleListTooSmall)
                };

                for group in assignment.insider_groups.clone() {
                    unsafe {group.add_player_to_revealed_group_unchecked(&mut game, player);}
                }

                let win_con = player.win_condition(&game).clone();
                player.set_win_condition(&mut game, win_con);
                InsiderGroups::send_player_insider_groups_packet(&game, player);
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
        
        Abilities::set_default_abilties(&mut game);

        //initial role creation calls "on role created". It acts as if your role was just switched but doesnt call on role switch
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
        OnGameStart::new().invoke(&mut game);

        Ok(game)
    }
    
    /// `assignment.assignments` must have length 255 or lower
    pub fn create_assignments(assignment: OutlineListAssignment)->Assignments{
        let mut assignments = Assignments::new();

        for outline_assignment in assignment.assignments {
            assignments.insert(outline_assignment.player, outline_assignment);
        }

        assignments
    }
}