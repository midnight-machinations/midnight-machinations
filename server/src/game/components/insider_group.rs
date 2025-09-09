use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::{game::{chat::{ChatGroup, ChatMessageVariant}, event::{on_add_insider::OnAddInsider, on_conceal_role::OnConcealRole, on_remove_insider::OnRemoveInsider, Event}, player::PlayerReference, Assignments, Game}, packet::ToClientPacket, vec_set::VecSet};

#[derive(Debug)]
pub struct InsiderGroups{
    mafia: InsiderGroup,
    cult: InsiderGroup,
    puppeteer: InsiderGroup,
    generic: HashMap<u8, InsiderGroup>
}
impl InsiderGroups{

    pub fn broken_new()->Self{
        Self{
            mafia: InsiderGroup::default(),
            cult: InsiderGroup::default(),
            puppeteer: InsiderGroup::default(),
            generic: HashMap::new(),
        }
    }

    /// # Safety
    /// player_count is correct
    /// assignments contains all players
    pub unsafe fn new(player_count: u8, assignments: &Assignments) -> Self {
        let mut out = Self {
            mafia: InsiderGroup::default(),
            cult: InsiderGroup::default(),
            puppeteer: InsiderGroup::default(),
            generic: HashMap::new()
        };
        for player in unsafe { PlayerReference::all_players_from_count(player_count) }{
            for group in assignments
                .get(&player)
                .expect("assignments is required to hold all players for safety")
                .insider_groups
                .iter()
                .copied()
            {
                if let Some(existing_group) = out.get_group_mut(group) {
                    existing_group.players.insert(player);
                    continue;
                } else if let InsiderGroupID::Generic { key } = group {
                    out.generic.insert(key, InsiderGroup { players: vec![player].into_iter().collect() });
                    continue;
                }
            }
        }
        out
    }


    
    pub fn on_conceal_role(game: &mut Game, event: &OnConcealRole, _fold: &mut (), _priority: ()){
        let &OnConcealRole{player, concealed_player} = event;
        InsiderGroupID::Mafia.on_conceal_role(game, player, concealed_player);
        InsiderGroupID::Cult.on_conceal_role(game, player, concealed_player);
        InsiderGroupID::Puppeteer.on_conceal_role(game, player, concealed_player);
    }
    // packets
    pub fn send_fellow_insiders_packets(game: &Game, player: PlayerReference){
        let fellow_insiders = PlayerReference::all_players(game)
            .filter(|p| InsiderGroupID::in_same_group(game, *p, player))
            .map(|p| p.index())
            .collect();

        player.send_packet(game, ToClientPacket::YourFellowInsiders{fellow_insiders});
    }
    pub fn send_player_insider_groups_packet(game: &Game, player: PlayerReference){
        let mut groups = VecSet::new();
        for group in InsiderGroupID::all(game){
            if group.contains_player(game, player){
                groups.insert(group);
            }
        }
        player.send_packet(game, ToClientPacket::YourInsiderGroups{insider_groups: groups});
    }

    fn get_group(&self, id: InsiderGroupID)->Option<&InsiderGroup>{
        match id {
            InsiderGroupID::Mafia => Some(&self.mafia),
            InsiderGroupID::Cult => Some(&self.cult),
            InsiderGroupID::Puppeteer => Some(&self.puppeteer),
            InsiderGroupID::Generic { key } => self.generic.get(&key)
        }
    }
    fn get_group_mut(&mut self, id: InsiderGroupID)->Option<&mut InsiderGroup>{
        match id {
            InsiderGroupID::Mafia => Some(&mut self.mafia),
            InsiderGroupID::Cult => Some(&mut self.cult),
            InsiderGroupID::Puppeteer => Some(&mut self.puppeteer),
            InsiderGroupID::Generic { key } => self.generic.get_mut(&key),
        }
    }

    pub fn get_generic_group_keys(&self) -> impl Iterator<Item = u8> + '_ {
        self.generic.keys().copied()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InsiderGroupID{
    Mafia,
    Cult,
    Puppeteer,
    #[serde(rename_all = "camelCase")]
    Generic { key: u8 }
}
#[derive(Default, Debug)]
pub struct InsiderGroup{
    players: VecSet<PlayerReference>
}

impl InsiderGroupID{
    //const
    pub fn all_static()->VecSet<InsiderGroupID>{
        vec![
            InsiderGroupID::Mafia,
            InsiderGroupID::Cult,
            InsiderGroupID::Puppeteer
        ].into_iter().collect()
    }
    
    pub fn all(game: &Game)->VecSet<InsiderGroupID>{
        let mut result = Self::all_static();
        for key in game.insider_groups.get_generic_group_keys() {
            result.insert(InsiderGroupID::Generic { key });
        }
        result
    }
    pub const fn get_insider_chat_group(&self)->ChatGroup{
        match self{
            InsiderGroupID::Mafia=>ChatGroup::Mafia,
            InsiderGroupID::Cult=>ChatGroup::Cult,
            InsiderGroupID::Puppeteer=>ChatGroup::Puppeteer,
            InsiderGroupID::Generic { key }=>ChatGroup::Generic { key: *key }
        }
    }
    pub fn get_insider_group_from_chat_group(game: &Game, chat: &ChatGroup)->Option<InsiderGroupID>{
        for inside in Self::all(game) {
            if inside.get_insider_chat_group() == *chat {
                return Some(inside)
            }
        }
        None
    }
    fn deref<'a>(&self, game: &'a Game)->Option<&'a InsiderGroup>{
        game.insider_groups.get_group(*self)
    }
    fn deref_mut<'a>(&self, game: &'a mut Game)->Option<&'a mut InsiderGroup>{
        game.insider_groups.get_group_mut(*self)
    }
    pub fn players(&self, game: &Game)->VecSet<PlayerReference>{
        self.deref(game).map(|group| group.players.clone()).unwrap_or_default()
    }
    
    pub fn reveal_group_players(&self, game: &mut Game){
        for a in self.players(game).clone() {
            InsiderGroups::send_fellow_insiders_packets(game, a);
            for b in self.players(game).clone() {
                a.reveal_players_role(game, b);
            }
        }
    }

    // Mutations
    /// # Safety
    /// This function will not alert the other players of the addition of this new player
    pub unsafe fn add_player_to_revealed_group_unchecked(&self, game: &mut Game, player: PlayerReference){
        if let Some(group) = self.deref_mut(game) {
            group.players.insert(player);
        } else {
            game.insider_groups.generic.insert(
                match self {
                    InsiderGroupID::Generic { key } => *key,
                    _ => unreachable!("Only generic groups can be None"),
                },
                InsiderGroup {
                    players: vec![player].into_iter().collect()
                }
            );
        }
        OnAddInsider::new(player, *self).invoke(game);
    }
    pub fn add_player_to_revealed_group(&self, game: &mut Game, player: PlayerReference){
        let should_reveal = {
            if let Some(group) = self.deref_mut(game) {
                group.players.insert(player).is_none()
            } else {
                if let InsiderGroupID::Generic { key } = self {
                    game.insider_groups.generic.insert(*key, InsiderGroup {
                        players: vec![player].into_iter().collect()
                    });
                }

                true
            }
        };

        if should_reveal {
            self.reveal_group_players(game);
        }
        OnAddInsider::new(player, *self).invoke(game);
        InsiderGroups::send_player_insider_groups_packet(game, player);
    }
    pub fn remove_player_from_insider_group(&self, game: &mut Game, player: PlayerReference){
        let should_reveal = {
            if let Some(group) = self.deref_mut(game) {
                group.players.remove(&player).is_some()
            } else {
                false // They must have not been in it, so no need to reveal
            }
        };

        if should_reveal {
            self.reveal_group_players(game);
        }
        OnRemoveInsider::new(player, *self).invoke(game);
        InsiderGroups::send_player_insider_groups_packet(game, player);
    }
    pub fn set_player_insider_groups(set: VecSet<InsiderGroupID>, game: &mut Game, player: PlayerReference){
        for group in InsiderGroupID::all(game){
            if set.contains(&group){
                group.add_player_to_revealed_group(game, player);
            }else{
                group.remove_player_from_insider_group(game, player);
            }
        }
    }

    // Events
    pub fn on_conceal_role(&self, game: &mut Game, player: PlayerReference, concealed_player: PlayerReference){
        if self.contains_player(game, concealed_player) && self.contains_player(game, player) {
            self.reveal_group_players(game);
        }
    }


    // Queries
    pub fn in_any_group(game: &Game, player: PlayerReference)->bool{
        InsiderGroupID::all(game).into_iter().any(|g|g.contains_player(game, player))
    }
    pub fn contains_player(&self, game: &Game, player: PlayerReference)->bool{
        let players: VecSet<PlayerReference> = self.players(game);
        players.contains(&player)
    }
    pub fn in_same_group(game: &Game, a: PlayerReference, b: PlayerReference)->bool{
        InsiderGroupID::all(game).iter().any(|group| group.contains_player(game, b) && group.contains_player(game, a))
    }
    pub fn all_groups_with_player(game: &Game, player_ref: PlayerReference)->VecSet<InsiderGroupID>{
        InsiderGroupID::all(game)
            .into_iter()
            .filter(|group| 
                group.contains_player(game, player_ref)
            ).collect()
    }
    

    

    //other
    pub fn send_message_in_available_insider_chat_or_private(
        game: &mut Game,
        player: PlayerReference,
        message: ChatMessageVariant,
        send_private_backup: bool
    ){
        let mut message_sent = false;
        for chat_group in player.get_current_send_chat_groups(game){
            if Self::get_insider_group_from_chat_group(game, &chat_group).is_none() {continue};
            game.add_message_to_chat_group(chat_group, message.clone());
            message_sent = true;
        }
        if !message_sent && send_private_backup {
            player.add_private_chat_message(game, message);
        }
    }
}