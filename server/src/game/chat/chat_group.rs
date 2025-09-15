use std::iter::once;

use serde::{Deserialize, Serialize};

use crate::{
    game::{
        components::insider_group::InsiderGroups,
        player::PlayerReference,
        Game
    }, vec_map::VecMap, vec_set::{vec_set, VecSet}
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ChatGroup {
    All,
    Dead,

    Mafia,
    Cult,

    Jail,
    Kidnapped,
    Interview,
    Puppeteer,
    Warden,
}


pub struct PlayerChatGroupMap(VecMap<PlayerReference, VecSet<ChatGroup>>);
impl PlayerChatGroupMap{
    pub fn new()->Self{
        Self(
            VecMap::new()
        )
    }
    pub fn combined(other: Vec<Self>)->Self{
        let mut out = Self::new();
        for other in other {
            for (player, other_chat_groups) in other.0 {
                if let Some(chat_groups) = out.0.get_mut(&player) {
                    chat_groups.extend(other_chat_groups);
                }else{
                    out.0.insert(player, other_chat_groups);
                }
            }
        }
        out
    }
    pub fn insert(&mut self, player: PlayerReference, chat_group: ChatGroup){
        if let Some(groups) = self.0.get_mut(&player){
            groups.insert(chat_group);
        }else{
            self.0.insert(player, vec_set![chat_group]);
        }
    }
    fn player_in_group(&self, player: PlayerReference, chat_group: ChatGroup)->bool{
        let Some(groups) = self.0.get(&player) else {return false};
        groups.contains(&chat_group)
    }

    pub fn receive_player_chat_group_map(game: &Game)->Self{
        Self::combined(
            PlayerReference::all_players(game)
                .map(|p|p.receive_player_chat_group_map(game))
                .chain(once(InsiderGroups::receive_player_chat_group_map(game)))
                .collect()
        )
    }
}

impl ChatGroup{
    pub fn all_players_in_group(&self, game: &Game)->Vec<PlayerReference>{
        let map = PlayerChatGroupMap::receive_player_chat_group_map(game);

        let mut out = Vec::new();
        for player_ref in PlayerReference::all_players(game){
            if map.player_in_group(player_ref, *self) {
                out.push(player_ref);
            }
        }
        out
    }
}
impl PlayerReference{
    fn receive_player_chat_group_map(&self, game: &Game)->PlayerChatGroupMap{
        let mut out = self.role_state(game).clone().receive_player_chat_group_map(game, *self);
        out.insert(*self, ChatGroup::All);
        if !self.alive(game) {out.insert(*self, ChatGroup::Dead);}
        out
    }
}