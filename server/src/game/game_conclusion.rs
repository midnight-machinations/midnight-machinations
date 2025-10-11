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
                .map(|p|GameOverCheckPlayer::from_player(game, p))
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
}
impl PlayerReference{
    /// If they can consistently kill then they keep the game running
    /// Town kills by voting
    /// Mafia kills with MK or gun
    /// Cult kills / converts
    pub fn keeps_game_running(&self, game: &Game) -> bool {
        GameOverCheckPlayer::from_player(game, *self).keeps_game_running()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct GameOverCheckPlayer{
    pub alive: bool,
    pub role: Role,
    pub insider_groups: VecSet<InsiderGroupID>,
    pub win_condition: WinCondition,
}
impl GameOverCheckPlayer{
    fn from_player(game: &Game, player: PlayerReference)->Self{
        Self{
            alive: player.alive(game),
            role: player.role(game),
            insider_groups: InsiderGroupID::all_groups_with_player(game, player),
            win_condition: player.win_condition(game).clone()
        }
    }
    /// *has the ability to change what the set of living players win conditions are until game over (convert, marionette, kill)*
    pub fn keeps_game_running(&self)->bool{
        if !self.alive {return false;}
        if self.insider_groups.contains(&InsiderGroupID::Mafia) {return true;}  //will get SK or gun
        if self.insider_groups.contains(&InsiderGroupID::Cult) {return true;}   //will get converted to apostle, so can kill
        if self.win_condition.is_loyalist_for(GameConclusion::Town) {return true;}  //"can vote", need this or else theres no other team
        
        //Role can kill / convert
        if
            RoleSet::Fiends.get_roles().contains(&self.role) ||
            RoleSet::MafiaKilling.get_roles().contains(&self.role) 
        {
            true
        }else{
            matches!(self.role, Role::Apostle | Role::Zealot | Role::Krampus | Role::Politician)
        }
    }
}