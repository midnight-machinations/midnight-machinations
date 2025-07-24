
use serde::{Deserialize, Serialize};
use crate::{game::{event::on_convert::OnConvert, game_conclusion::GameConclusion, player::PlayerReference, role_list::RoleAssignment, role_outline_reference::RoleOutlineReference, Game}, vec_map::VecMap, vec_set::{vec_set, VecSet}};

use super::player_component::PlayerComponent;

impl PlayerComponent::<WinCondition>{
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8, assignments: &VecMap<PlayerReference, (RoleOutlineReference,RoleAssignment)>)->Self{
        PlayerComponent::<WinCondition>::new_component_box(
            num_players,
            |player|assignments.get(&player).expect("Already checked this was fine").1.win_condition()
        )
    }
}
impl PlayerReference{
    pub fn win_condition<'a>(&self, game: &'a Game) -> &'a WinCondition {
        game.win_condition.get(*self)
    }
    pub fn set_win_condition(&self, game: &mut Game, win_condition: WinCondition){
        let old_win_condition = self.win_condition(game).clone();
        *game.win_condition.get_mut(*self) = win_condition.clone();

        OnConvert::new(*self, old_win_condition, win_condition).invoke(game)
    }
}



/// Related functions require RoleStateWon to be independent of GameConclusion. 
/// RoleStateWon needs to be able to win with any GameConclusion.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum WinCondition{
    #[serde(rename_all = "camelCase")]
    GameConclusionReached{
        win_if_any: VecSet<GameConclusion>
    },
    RoleStateWon,
}

impl PartialOrd for WinCondition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WinCondition {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}



impl WinCondition{
    pub fn required_resolution_states_for_win(&self)->Option<VecSet<GameConclusion>>{
        match self{
            WinCondition::GameConclusionReached{win_if_any} => Some(win_if_any.clone()),
            WinCondition::RoleStateWon => None,
        }
    }
    pub fn are_friends(a: &WinCondition, b: &WinCondition)->bool{
        let a_conditions = a.required_resolution_states_for_win();
        let b_conditions = b.required_resolution_states_for_win();

        match (a_conditions, b_conditions){
            (Some(a), Some(b)) => a.intersection(&b).count() > 0,
            _ => true
        }
    }
    pub fn friends_with_resolution_state(&self, resolution_state: GameConclusion)->bool{
        match self{
            WinCondition::GameConclusionReached{win_if_any} => win_if_any.contains(&resolution_state),
            WinCondition::RoleStateWon => true,
        }
    }
    pub fn is_loyalist_for(&self, resolution_state: GameConclusion)->bool{
        match self{
            WinCondition::GameConclusionReached{win_if_any} => win_if_any.count() == 1 && win_if_any.contains(&resolution_state),
            WinCondition::RoleStateWon => false,
        }
    }
    pub fn is_loyalist(&self)->bool{
        match self{
            WinCondition::GameConclusionReached{win_if_any} => win_if_any.count() == 1,
            WinCondition::RoleStateWon => false,
        }
    }
    
    pub fn new_loyalist(resolution_state: GameConclusion) -> WinCondition {
        WinCondition::GameConclusionReached { win_if_any: vec_set![resolution_state] }
    }
    pub fn is_role_state(&self) -> bool {
        matches!(self, Self::RoleStateWon)
    }
}