use serde::{Deserialize, Serialize};
use crate::{game::{event::{on_convert::OnConvert, Event}, game_conclusion::GameConclusion, player::PlayerReference, role::{chronokaiser::Chronokaiser, RoleState}, Assignments, Game}, vec_set::{vec_set, VecSet}};

use super::player_component::PlayerComponent;
pub type WinConditionComponent = PlayerComponent::<WinCondition>;
impl WinConditionComponent{
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8, assignments: &Assignments) -> Self {
        unsafe {
            PlayerComponent::<WinCondition>::new_component_box(
                num_players,
                |player| assignments.get(&player).expect("Already checked this was fine").win_condition.clone()
            )
        }
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
    pub fn win_if_any_conclusions(&self)->Option<VecSet<GameConclusion>>{
        match self{
            WinCondition::GameConclusionReached{win_if_any} => Some(win_if_any.clone()),
            WinCondition::RoleStateWon => None,
        }
    }
    pub fn are_friends(a: &WinCondition, b: &WinCondition)->bool{
        let a_conditions = a.win_if_any_conclusions();
        let b_conditions = b.win_if_any_conclusions();

        match (a_conditions, b_conditions){
            (Some(a), Some(b)) => a.intersection(&b).count() > 0,
            _ => true
        }
    }
    pub fn friends_with_conclusion(&self, resolution_state: GameConclusion)->bool{
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

    pub fn won_with_role_state(game: &Game, player: PlayerReference)->bool{
        match player.role_state(game) {
            RoleState::Jester(r) => r.won(),
            RoleState::Doomsayer(r) => r.won(),
            RoleState::Mercenary(r) => r.won(),
            RoleState::Revolutionary(r) => r.won(),
            RoleState::Chronokaiser(_) => Chronokaiser::won(game, player),
            RoleState::Martyr(r) => r.won(),
            _ => false
        }
    }
}

impl PlayerReference{
    pub fn get_won_game(&self, game: &Game, conclusion: GameConclusion) -> bool {
        match self.win_condition(game){
            WinCondition::GameConclusionReached { win_if_any } => win_if_any.contains(&conclusion),
            WinCondition::RoleStateWon => WinCondition::won_with_role_state(game, *self),
        }
    }
}