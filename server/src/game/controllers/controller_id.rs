use serde::{Deserialize, Serialize};

use crate::game::{
    controllers::Controllers, modifiers::hidden_nomination_votes::HiddenNominationVotes, player::PlayerReference, role::Role, Game
};

use super::{
    ControllerSelection, BooleanSelection,
    Controller, StringSelection,
};

pub type RoleControllerID = u8;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag="type")]
pub enum ControllerID{
    CallWitness{player: PlayerReference},
    Nominate{player: PlayerReference},
    Judge{player: PlayerReference},
    
    Chat{player: PlayerReference},
    ChatIsBlock{player: PlayerReference},
    SendChat{player: PlayerReference},

    Whisper{player: PlayerReference},
    WhisperToPlayer{player: PlayerReference},
    SendWhisper{player: PlayerReference},

    Alibi{player: PlayerReference},

    #[serde(rename_all = "camelCase")]
    Role{
        player: PlayerReference,
        role: Role,
        id: RoleControllerID
    },
    ForfeitNominationVote{player: PlayerReference},
    PitchforkVote{player: PlayerReference},
    

    ForwardMessage{player: PlayerReference},


    SyndicateGunShoot,
    SyndicateGunGive,
    SyndicateChooseBackup,
    SyndicateBackupAttack,

    WardenCooperate{
        warden: PlayerReference,
        player: PlayerReference,
    }
}
impl ControllerID{
    pub fn role(player: PlayerReference, role: Role, id: RoleControllerID)->Self{
        Self::Role{player, role, id}
    }
    pub fn nominate(player: PlayerReference)->Self{
        Self::Nominate{player}
    }
    pub fn judge(player: PlayerReference)->Self{
        Self::Judge{player}
    }
    pub fn chat(player: PlayerReference)->Self{
        Self::Chat{player}
    }
    pub fn chat_is_block(player: PlayerReference)->Self{
        Self::ChatIsBlock{player}
    }
    pub fn whisper(player: PlayerReference)->Self{
        Self::Whisper{player}
    }
    pub fn whisper_to_player(player: PlayerReference)->Self{
        Self::WhisperToPlayer{player}
    }
    pub fn forfeit_vote(player: PlayerReference)->Self{
        Self::ForfeitNominationVote{player}
    }
    pub fn pitchfork_vote(player: PlayerReference)->Self{
        Self::PitchforkVote{player}
    }
    pub fn syndicate_gun_item_shoot()->Self{
        Self::SyndicateGunShoot
    }
    pub fn syndicate_gun_item_give()->Self{
        Self::SyndicateGunGive
    }
    pub fn syndicate_choose_backup()->Self{
        Self::SyndicateChooseBackup
    }
    pub fn syndicate_backup_attack()->Self{
        Self::SyndicateBackupAttack
    }
}


impl ControllerID{
    pub fn current_used_ids(game: &Game)->Box<[Self]>{
        game.controllers.controllers.iter().map(|(id,_)|id).cloned().collect()
    }
    pub fn should_send_selection_chat_message(&self, game: &Game)->bool{
        if 
            matches!(self, ControllerID::Nominate { .. }) &&
            HiddenNominationVotes::nomination_votes_are_hidden(game)
        {
            true
        }else{
            !matches!(self, 
                ControllerID::Nominate { .. } | 
                ControllerID::ForwardMessage { .. } |
                ControllerID::Alibi { .. } |
                ControllerID::Chat { .. } |
                ControllerID::ChatIsBlock { .. } |
                ControllerID::SendChat { .. } |
                ControllerID::Whisper { .. } |
                ControllerID::WhisperToPlayer { .. } |
                ControllerID::SendWhisper { .. }
            )
        }
    }
    fn get_controller<'a>(&self, game: &'a Game)->Option<&'a Controller>{
        game.controllers.controllers.get(self)
    }
    pub fn get_selection<'a>(&self, game: &'a Game)->Option<&'a ControllerSelection>{
        let saved_controller = self.get_controller(game)?;
        Some(saved_controller.selection())
    }


    pub fn get_boolean_selection<'a>(&self, game: &'a Game)->Option<&'a BooleanSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::Boolean(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }
    pub fn get_string_selection<'a>(&self, game: &'a Game)->Option<&'a StringSelection>{
        self.get_selection(game)
            .and_then(|selection| 
                if let ControllerSelection::String(selection) = selection {
                    Some(selection)
                }else{
                    None
                }
            )
    }

    pub fn set_selection(self, game: &mut Game, actor: Option<PlayerReference>, selection: impl Into<ControllerSelection>, overwrite_gray_out: bool){
        Controllers::set_selection_in_controller(
            game,
            actor,
            self,
            selection,
            overwrite_gray_out
        );
    }
}