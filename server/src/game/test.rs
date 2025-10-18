use rand::{rngs::SmallRng, SeedableRng as _};

use crate::{game::{abilities_component::Abilities, chat::{ChatComponent, PlayerChatGroups}, components::{blocked::BlockedComponent, enfranchise::EnfranchiseComponent, fast_forward::FastForwardComponent, graves::Graves, pitchfork_item::PitchforkItemComponent, role::RoleComponent, role_reveal::RevealedPlayersComponent}, event::Event, role_list_generation::RoleListGenerator}, vec_map::VecMap};

use super::{
    controllers::Controllers, components::{
        cult::Cult, fragile_vest::FragileVests, insider_group::InsiderGroups,
        mafia::Mafia, mafia_recruits::MafiaRecruits, player_component::PlayerComponent,
        poison::Poison, puppeteer_marionette::PuppeteerMarionette, silenced::Silenced,
        synopsis::SynopsisTracker, tags::Tags, verdicts_today::VerdictsToday, win_condition::WinCondition
    }, event::on_game_start::OnGameStart,
    phase::PhaseStateMachine, player::{test::mock_player, PlayerReference},
    settings::Settings, Assignments, Game, RejectStartReason
};

pub fn mock_game(settings: Settings, num_players: u8) -> Result<(Game, Assignments), RejectStartReason> {

    //check settings are not completly off the rails
    if settings.phase_times.game_ends_instantly() {
        return Err(RejectStartReason::ZeroTimeGame);
    }
    let mut rng = SmallRng::seed_from_u64(settings.random_seed.unwrap_or_else(rand::random));

    let settings = settings.clone();

    let mut role_list_generator = RoleListGenerator::new(settings.clone(), &mut rng);

    let random_outline_assignments = match role_list_generator.generate_role_list() {
        Some(roles) => {roles},
        None => {return Err(RejectStartReason::RoleListCannotCreateRoles);}
    };

    let assignments = Game::create_assignments(random_outline_assignments);

    let mut players = Vec::new();
    for player in unsafe{PlayerReference::all_players_from_count(num_players)} {
        let new_player = mock_player(
            format!("{}",player.index())
        );
        players.push(new_player);
    }

    let mut game = Game{
        clients: VecMap::new(),
        room_name: "Test".to_string(),
        // pitchfork: Pitchfork::new(num_players),
        
        assignments: assignments.clone(),
        ticking: true,
        rng: rng.clone(),
        spectators: Vec::new(),
        spectator_chat_messages: Vec::new(),
        players: players.into_boxed_slice(),
        phase_machine: PhaseStateMachine::new(settings.phase_times.clone()),
        settings,

        player_chat_groups: PlayerChatGroups::new(),
        enfranchise: unsafe{EnfranchiseComponent::new(num_players)},
        abilities: Abilities::new(&assignments),
        graves: Graves::default(),
        controllers: Controllers::default(),
        cult: Cult::default(),
        mafia: Mafia,
        puppeteer_marionette: PuppeteerMarionette::default(),
        mafia_recruits: MafiaRecruits::default(),
        verdicts_today: VerdictsToday::default(),
        blocked: unsafe {BlockedComponent::new(num_players)},
        poison: Poison::default(),
        insider_groups: unsafe{InsiderGroups::new(num_players, &assignments)},
        detained: Default::default(),
        confused: Default::default(),
        synopsis_tracker: SynopsisTracker::new(num_players),
        tags: Tags::default(),
        silenced: Silenced::default(),
        role: unsafe{RoleComponent::new(num_players, &assignments)},
        pitchfork_item: unsafe{PitchforkItemComponent::new(num_players)},
        fragile_vests: unsafe{PlayerComponent::<FragileVests>::new(num_players)},
        win_condition: unsafe{PlayerComponent::<WinCondition>::new(num_players, &assignments)},
        fast_forward: unsafe{FastForwardComponent::new(num_players)},
        chat_messages: unsafe{ChatComponent::new(num_players)},
        revealed_players: unsafe{RevealedPlayersComponent::new(num_players)}
    };


    //set wincons
    for player in PlayerReference::all_players(&game){
        let role: super::role::Role = player.role(&game);
        let role_data = role.new_state(&mut game);
        player.set_win_condition(&mut game, role_data.clone().default_win_condition());
        InsiderGroups::send_player_insider_groups_packet(&game, player);
    }
    
    Abilities::set_default_abilties(&mut game);

    //on role creation needs to be called after all players roles are known
    for player_ref in PlayerReference::all_players(&game){
        player_ref.initial_set_role_insider_wincondition(&mut game);
    }

    OnGameStart::new().invoke(&mut game);

    Ok((game, assignments))
}