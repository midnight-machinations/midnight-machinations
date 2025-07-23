use mafia_server::game::{
    chat::ChatMessageVariant, 
    player::PlayerReference, 
    settings::Settings, 
    test::mock_game, 
    Game
};

pub mod player;
pub mod game;

pub struct TestScenario {
    pub game: Game,
    pub players: Vec<PlayerReference>
}

#[allow(unused)]
macro_rules! scenario {
    ($game:ident in Briefing 1 $($tok:tt)*) => {
        kit::scenario!($game $($tok)*);
    };
    ($game:ident in $phase:ident $day:literal $($tok:tt)*) => {
        kit::scenario!($game $($tok)*);
        $game.skip_to(PhaseType::$phase, $day);
    };
    ($game:ident where
        $($name:ident: $role:ident),*
    ) => {
        let mut scenario = kit::_init::create_basic_scenario(
            // vec![$(RoleState::$role($role::default())),*]
            vec![$(Role::$role),*]
        );

        let game = &mut scenario.game;

        let players: Vec<kit::player::TestPlayer> = scenario.players
            .into_iter()
            .map(|player| kit::player::TestPlayer::new(player, &game))
            .collect();

        let [$($name),*] = players.as_slice() else {unreachable!()};

        let mut $game = kit::game::TestGame::new(game);
        $(let $name = *$name;)*
    }
}

#[allow(unused)]
macro_rules! assert_contains {
    ($container:expr, $value:expr) => {
        assert!($container.contains(&$value), "{}", format!("Expected {:#?} to contain {:?}", $container, $value));
    };
}
#[allow(unused)]
macro_rules! assert_not_contains {
    ($container:expr, $value:expr) => {
        assert!(!$container.contains(&$value), "{}", format!("Expected {:#?} not to contain {:?}", $container, $value))
    };
}

#[allow(unused)]
pub(crate) use {scenario, assert_contains, assert_not_contains};

//Formats messages in a way where it's clear which phase each message was sent in
pub fn _format_messages_debug(messages: Vec<ChatMessageVariant>) -> String{
    let mut string = "[\n".to_string();

    for message in messages {
        string += match message {
            ChatMessageVariant::PhaseChange{..} => "\t",
            _ => "\t\t",
        };
        string += format!("{:?}", message).as_str();
        string += "\n";
    }
    string += "]";
    string
}

/// Stuff that shouldn't be called directly - only in macro invocations.
#[doc(hidden)]
pub mod _init {
    use mafia_server::{game::{
        role::Role,
        role_list::{
            RoleList, RoleOutline, RoleOutlineOption, RoleOutlineOptionInsiderGroups,
            RoleOutlineOptionRoles, RoleOutlineOptionWinCondition
        }
    }, vec_set::VecSet};
    use vec1::vec1;

    use super::*;

    pub fn create_basic_scenario(roles: Vec<Role>) -> TestScenario {
        let mut role_list = Vec::new();
        for (i, role) in roles.iter().enumerate() {
            role_list.push(RoleOutline { options: 
                vec1![RoleOutlineOption {
                    roles: RoleOutlineOptionRoles::Role { role: *role },
                    insider_groups: RoleOutlineOptionInsiderGroups::RoleDefault,
                    win_condition: RoleOutlineOptionWinCondition::RoleDefault,
                    player_pool: VecSet::from_iter(vec![i as u8]),
                }]
            });
        }
    
        let (game, mut assignments) = match mock_game(Settings {
            role_list: RoleList(role_list),
            enabled_roles: Role::values().into_iter().collect(),
            ..Default::default()
        }, roles.len() as u8){
            Ok(game) => game,
            Err(err) => panic!("Failed to create game: {:?}", err),
        };

        let mut players_out: Vec<PlayerReference> = PlayerReference::all_players(&game).collect();

        //reorder players to be in the same order as roleoutline
        for player in players_out.iter_mut(){
            let role = roles.get(player.index() as usize)
                .expect("test scenario assert");
            let found_player = *assignments
                .iter()
                .find(|(_,(_,r))|r.role() == *role)
                .map(|(p,_)|p)
                .expect("test scenario assert");

            assignments.remove(&found_player);
            *player = found_player;
        }
    
        TestScenario { game, players: players_out }
    }
}
