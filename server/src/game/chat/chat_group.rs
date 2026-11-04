use std::iter::once;

use serde::{Deserialize, Serialize};

use crate::game::controllers::{ControllerID, ControllerSelection};

use crate::{
    game::{
        Game, abilities_component::Abilities, components::{call_witness::CallWitness, insider_group::InsiderGroups, silenced::Silenced}, modifiers::ModifierID, phase::PhaseType, player::PlayerReference
    }, packet::ToClientPacket, vec_map::VecMap, vec_set::{VecSet, vec_set}
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
#[derive(PartialEq, Eq)]
pub struct PlayerChatGroups{
    send: PlayerChatGroupMap,
}
impl PlayerChatGroups{
    pub fn new()->Self{Self { send: PlayerChatGroupMap::new() }}
    pub fn send_player_chat_group_map(game: &mut Game)->PlayerChatGroupMap{
        let out = Self::compute_send_map(game);

        if game.player_chat_groups.send != out {
            for player in PlayerReference::all_players(game){
                player.send_packet(game, ToClientPacket::YourSendChatGroups { send_chat_groups: out.get(player)});
            }
        }

        game.player_chat_groups.send = out.clone();

        out
    }
    /// Shared permission calculation for text and voice, without sending packets.
    pub fn compute_send_map(game: &Game) -> PlayerChatGroupMap {
        if game.modifier_settings().is_enabled(ModifierID::NoChat) {return PlayerChatGroupMap::new();}
        PlayerChatGroupMap::combine(
            PlayerReference::all_players(game)
                .map(|p|p.send_player_chat_group_map(game))
                .chain(once(Abilities::send_player_chat_group_map(game)))
                .chain(once(InsiderGroups::send_player_chat_group_map(game)))
                .chain(once(CallWitness::send_player_chat_group_map(game)))
        )
    }
    pub fn receive_player_chat_group_map(game: &Game)->PlayerChatGroupMap{
        PlayerChatGroupMap::combine(
            PlayerReference::all_players(game)
                .map(|p|p.receive_player_chat_group_map(game))
                .chain(once(Abilities::receive_player_chat_group_map(game)))
                .chain(once(InsiderGroups::receive_player_chat_group_map(game)))
        )
    }
}
#[derive(Clone, PartialEq, Eq, Default)]
pub struct PlayerChatGroupMap(VecMap<PlayerReference, VecSet<ChatGroup>>);
impl PlayerChatGroupMap{
    pub fn new()->Self{
        Self(
            VecMap::new()
        )
    }
    pub fn combine(other: impl Iterator<Item = Self>)->Self{
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
    pub(crate) fn get(&self, player: PlayerReference)->VecSet<ChatGroup>{
        if let Some(out) = self.0.get(&player) {
            out.clone()
        }else{
            VecSet::new()
        }
    }
    fn player_in_group(&self, player: PlayerReference, chat_group: ChatGroup)->bool{
        let Some(groups) = self.0.get(&player) else {return false};
        groups.contains(&chat_group)
    }
}

impl ChatGroup{
    pub fn select_available(groups: &VecSet<Self>, selected: Option<Self>) -> Option<Self> {
        selected.filter(|group| groups.contains(group))
            .or_else(|| groups.contains(&Self::All).then_some(Self::All))
            .or_else(|| groups.iter().next().copied())
    }

    pub fn all_players_in_group(&self, game: &Game)->Vec<PlayerReference>{
        let map = PlayerChatGroups::receive_player_chat_group_map(game);

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
    /// The same destination is used for text and voice, including permission changes
    /// that happen before the next controller refresh.
    pub fn selected_send_chat_group(&self, game: &Game) -> Option<ChatGroup> {
        let selected = match (ControllerID::ChatGroup { player: *self }).get_selection(game) {
            Some(ControllerSelection::ChatGroup(selection)) => selection.0,
            _ => None,
        };
        let groups = PlayerChatGroups::compute_send_map(game).get(*self);
        ChatGroup::select_available(&groups, selected)
    }

    fn receive_player_chat_group_map(&self, game: &Game)->PlayerChatGroupMap{
        if game.current_phase().phase() == PhaseType::Recess {
            let mut out = PlayerChatGroupMap::new();
            out.insert(*self, ChatGroup::All);
            return out;
        }

        let mut out = PlayerChatGroupMap::default();
        out.insert(*self, ChatGroup::All);
        if !self.alive(game) {out.insert(*self, ChatGroup::Dead);}
        out
    }
    fn send_player_chat_group_map(&self, game: &Game)->PlayerChatGroupMap{
        if game.current_phase().phase() == PhaseType::Recess {
            let mut out = PlayerChatGroupMap::new();
            out.insert(*self, ChatGroup::All);
            return out;
        }

        if Silenced::silenced(game, *self) {
            return PlayerChatGroupMap::new();
        }

        let mut out = PlayerChatGroupMap::default();

        if 
            !matches!(game.current_phase().phase(), PhaseType::Night|PhaseType::Obituary|PhaseType::Testimony|PhaseType::Briefing|PhaseType::Recess) && 
            self.alive(game)
        {
            out.insert(*self, ChatGroup::All);
        }
        if !self.alive(game) {
            out.insert(*self, ChatGroup::Dead);
        }
        
        out
    }

    pub fn get_current_send_chat_groups(&self, game: &mut Game) -> VecSet<ChatGroup> {
        PlayerChatGroups::send_player_chat_group_map(game).get(*self)
    }
}
