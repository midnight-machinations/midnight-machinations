use serde::{Deserialize, Serialize};

use super::{components::win_condition::WinCondition, player::PlayerReference, role::Role, role_list::RoleSet, Game};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum GameConclusion {
    Town,
    Mafia,
    Cult,

    Fiends,

    Politician,

    NiceList,
    NaughtyList,

    Draw
}
impl GameConclusion {
    pub fn all()->Vec<GameConclusion>{
        vec![
            GameConclusion::Town,
            GameConclusion::Mafia,
            GameConclusion::Cult,

            GameConclusion::Fiends,

            GameConclusion::Politician,

            GameConclusion::NiceList,
            GameConclusion::NaughtyList,

            GameConclusion::Draw
        ]
    }
    ///either return Some(EndGameCondition) or None (if the game is not over yet)
    pub fn game_is_over(game: &Game)->Option<GameConclusion> {

        //Special wildcard case
        let living_roles = PlayerReference::all_players(game).filter_map(|player|{
            if player.alive(game){
                Some(player.role(game))
            }else{
                None
            }
        }).collect::<Vec<_>>();

        if living_roles.iter().all(|role|matches!(role, Role::Wildcard|Role::TrueWildcard)) && living_roles.len() > 1 {
            return None;
        }
        
        //if nobody is left to hold game hostage
        if !PlayerReference::all_players(game).any(|player| player.alive(game) && player.keeps_game_running(game)){
            return Some(GameConclusion::Draw);
        }

        //find one end game condition that everyone agrees on
        GameConclusion::all().into_iter().find(|resolution| 
            PlayerReference::all_players(game)
                .filter(|p|p.alive(game))
                .filter(|p|p.keeps_game_running(game))
                .all(|p|
                    match p.win_condition(game){
                        WinCondition::GameConclusionReached{win_if_any} => win_if_any.contains(resolution),
                        WinCondition::RoleStateWon => true,
                    }
                )
        )
    }

    pub fn get_premature_conclusion(game: &Game) -> GameConclusion {
        GameConclusion::game_is_over(game).unwrap_or(GameConclusion::Draw)
    }
    

    ///Town, Mafia, Cult, NK
    /// Has the ability to consistently kill till the end of the game
    /// *has the ability to change what the set of living players win conditions are until game over (convert, marionette, kill)*
    /// A detective and a witch game never ends so this needs to make sure they dont keep the game running
    /// For simplicity, i will just say only fiends, MK, apostle and zealot keep the game running
    pub fn keeps_game_running(role: Role)->bool{
        if
            RoleSet::Fiends.get_roles().contains(&role) ||
            RoleSet::MafiaKilling.get_roles().contains(&role)  
        {
            true
        }else{
            matches!(role, Role::Apostle | Role::Zealot | Role::Krampus)
        }
    }
}


//Endgamecondition -> One single game ending condition, if only these roles are left, the game ends
//Town, Mafia, Cult, Fiends, Politcian
//Victory condition -> If this is the endgamecondition of the game, you win
