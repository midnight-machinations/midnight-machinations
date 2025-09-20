use serde::{Deserialize, Serialize};

use crate::{game::components::insider_group::InsiderGroupID, vec_set::VecSet};

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
    pub fn game_is_over_game(game: &Game)->Option<GameConclusion> {
        Self::game_is_over(
            PlayerReference::all_players(game)
                .filter_map(|player|
                    if player.alive(game) {
                        Some(GameOverCheckPlayer{
                            role: player.role(game),
                            insider_groups: InsiderGroupID::all_groups_with_player(game, player),
                            win_condition: player.win_condition(game).clone()
                        })
                    }else{
                        None
                    }
                )
                .collect()
        )
    }

    pub fn game_is_over(players: Vec<GameOverCheckPlayer>)->Option<GameConclusion>{
        //Special wildcard case
        if
            players.iter().all(|player|matches!(player.role, Role::Wildcard|Role::TrueWildcard)) && 
            players.len() >= 2
        {
            return None;
        }
        
        //if nobody is left to hold game hostage
        if !players.iter().any(|player| player.keeps_game_running()){
            return Some(GameConclusion::Draw);
        }

        //find one end game condition that everyone agrees on
        GameConclusion::all().into_iter().find(|resolution| 
            players
                .iter()
                .filter(|p|p.keeps_game_running())
                .all(|p|p.win_condition.friends_with_conclusion(*resolution))
        )
    }

    pub fn get_premature_conclusion(game: &Game) -> GameConclusion {
        GameConclusion::game_is_over_game(game).unwrap_or(GameConclusion::Draw)
    }
    

    ///Town, Mafia, Cult, NK
    /// Has the ability to consistently kill till the end of the game
    /// *has the ability to change what the set of living players win conditions are until game over (convert, marionette, kill)*
    /// A detective and a witch game never ends so this needs to make sure they dont keep the game running
    /// For simplicity, i will just say only fiends, MK, apostle and zealot keep the game running
    pub fn keeps_game_running(role: Role)->bool{
        if
            RoleSet::Fiends.get_roles_static().contains(&role) ||
            RoleSet::MafiaKilling.get_roles_static().contains(&role) 
        {
            true
        }else{
            matches!(role, Role::Apostle | Role::Zealot | Role::Krampus | Role::Politician)
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct GameOverCheckPlayer{
    pub role: Role,
    pub insider_groups: VecSet<InsiderGroupID>,
    pub win_condition: WinCondition,
}
impl GameOverCheckPlayer{
    pub fn keeps_game_running(&self)->bool{
        if self.insider_groups.contains(&InsiderGroupID::Mafia) {return true;}
        if self.insider_groups.contains(&InsiderGroupID::Cult) {return true;}
        if self.win_condition.is_loyalist_for(GameConclusion::Town) {return true;}
        
        GameConclusion::keeps_game_running(self.role)
    }
}