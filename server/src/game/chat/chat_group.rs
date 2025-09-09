use serde::{Deserialize, Serialize};

use crate::game::{player::PlayerReference, Game};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(tag = "type", rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
    Generic { key: u8 },
}
impl ChatGroup{
    pub fn player_receive_from_chat_group(&self, game: &Game, player_ref: PlayerReference)->bool{
        player_ref.get_current_receive_chat_groups(game).contains(self)
    }

    pub fn all_players_in_group(&self, game: &Game)->Vec<PlayerReference>{
        let mut out = Vec::new();
        for player_ref in PlayerReference::all_players(game){
            if self.player_receive_from_chat_group(game, player_ref){
                out.push(player_ref);
            }
        }
        out
    }
}