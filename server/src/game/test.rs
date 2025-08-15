
    use crate::{game::{chat::ChatComponent, components::graves::Graves, role_list_generation::RoleListGenerator}, vec_map::VecMap};

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

        let mut role_list_generator = RoleListGenerator::new(&role_list);

        let random_outline_assignments = match role_list_generator.generate_role_list() {
            Some(roles) => {roles},
            None => {return Err(RejectStartReason::RoleListCannotCreateRoles);}
        };

        let assignments = Game::assign_players_to_roles(random_outline_assignments);

        let mut players = Vec::new();
        for player in unsafe{PlayerReference::all_players_from_count(num_players)} {
            let new_player = mock_player(
                format!("{}",player.index()),
                match assignments.get(&player).map(|a|a.1.role){
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
            phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
            settings,

            graves: Graves::default(),
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
            win_condition: unsafe{PlayerComponent::<WinCondition>::new(num_players, &assignments)},
            chat_messages: unsafe{ChatComponent::new(num_players)}
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