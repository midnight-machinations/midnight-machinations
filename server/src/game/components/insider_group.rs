use serde::{Deserialize, Serialize};

use crate::{game::{chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap}, event::{on_add_insider::OnAddInsider, on_conceal_role::OnConcealRole, on_remove_insider::OnRemoveInsider, Event}, player::PlayerReference, Assignments, Game}, packet::ToClientPacket, vec_set::VecSet};

#[derive(Debug)]
pub struct InsiderGroups{
    mafia: InsiderGroup,
    cult: InsiderGroup,
    puppeteer: InsiderGroup
}
impl InsiderGroups{
    /// # Safety
    /// player_count is correct
    /// assignments contains all players
    pub unsafe fn new(player_count: u8, assignments: &Assignments) -> Self {
        let mut out = Self {
            mafia: InsiderGroup::default(),
            cult: InsiderGroup::default(),
            puppeteer: InsiderGroup::default()
        };
        for player in unsafe { PlayerReference::all_players_from_count(player_count) }{
            for group in assignments
                .get(&player)
                .expect("assignments is required to hold all players for safety")
                .insider_groups
                .iter()
                .copied()
            {
                out.get_group_mut(group).players.insert(player);
            }
        }
        out
    }

    pub fn receive_player_chat_group_map(game: &Game)->PlayerChatGroupMap{
        let mut out = PlayerChatGroupMap::new();
        for group in InsiderGroupID::all(){
            for player in group.players(game).iter() {
                out.insert(*player, group.get_insider_chat_group());
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
            .collect();

        player.send_packet(game, ToClientPacket::YourFellowInsiders{fellow_insiders});
    }
    pub fn send_player_insider_groups_packet(game: &Game, player: PlayerReference){
        let mut groups = VecSet::new();
        for group in InsiderGroupID::all(){
            if group.contains_player(game, player){
                groups.insert(group);
            }
        }
        player.send_packet(game, ToClientPacket::YourInsiderGroups{insider_groups: groups});
    }

    fn get_group(&self, id: InsiderGroupID)->&InsiderGroup{
        match id {
            InsiderGroupID::Mafia => &self.mafia,
            InsiderGroupID::Cult => &self.cult,
            InsiderGroupID::Puppeteer => &self.puppeteer,
        }
    }
    fn get_group_mut(&mut self, id: InsiderGroupID)->&mut InsiderGroup{
        match id {
            InsiderGroupID::Mafia => &mut self.mafia,
            InsiderGroupID::Cult => &mut self.cult,
            InsiderGroupID::Puppeteer => &mut self.puppeteer,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InsiderGroupID{
    Mafia,
    Cult,
    Puppeteer
}
#[derive(Default, Debug)]
pub struct InsiderGroup{
    players: VecSet<PlayerReference>
}

impl InsiderGroupID{
    //const
    pub fn all()->VecSet<InsiderGroupID>{
        vec![
            InsiderGroupID::Mafia,
            InsiderGroupID::Cult,
            InsiderGroupID::Puppeteer
        ].into_iter().collect()
    }
    pub const fn get_insider_chat_group(&self)->ChatGroup{
        match self{
            InsiderGroupID::Mafia=>ChatGroup::Mafia,
            InsiderGroupID::Cult=>ChatGroup::Cult,
            InsiderGroupID::Puppeteer=>ChatGroup::Puppeteer
        }
    }
    pub fn get_insider_group_from_chat_group(chat: &ChatGroup)->Option<InsiderGroupID>{
        for inside in Self::all() {
            if inside.get_insider_chat_group() == *chat {
                return Some(inside)
            }
        }
        None
    }
    fn deref<'a>(&self, game: &'a Game)->&'a InsiderGroup{
        game.insider_groups.get_group(*self)
    }
    fn deref_mut<'a>(&self, game: &'a mut Game)->&'a mut InsiderGroup{
        game.insider_groups.get_group_mut(*self)
    }
    pub fn players<'a>(&self, game: &'a Game)->&'a VecSet<PlayerReference>{
        &self.deref(game).players
    }
    pub fn players_mut<'a>(&self, game: &'a mut Game)->&'a mut VecSet<PlayerReference>{
        &mut self.deref_mut(game).players
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
        self.players_mut(game).insert(player);
        OnAddInsider::new(player, *self).invoke(game);
    }
    pub fn add_player_to_revealed_group(&self, game: &mut Game, player: PlayerReference){
        if self.players_mut(game).insert(player).is_none() {
            self.reveal_group_players(game);
        }
        OnAddInsider::new(player, *self).invoke(game);
        InsiderGroups::send_player_insider_groups_packet(game, player);
    }
    pub fn remove_player_from_insider_group(&self, game: &mut Game, player: PlayerReference){
        if self.players_mut(game).remove(&player).is_some() {
            self.reveal_group_players(game);
        }
        OnRemoveInsider::new(player, *self).invoke(game);
        InsiderGroups::send_player_insider_groups_packet(game, player);
    }
    pub fn set_player_insider_groups(set: VecSet<InsiderGroupID>, game: &mut Game, player: PlayerReference){
        for group in InsiderGroupID::all(){
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
        InsiderGroupID::all().into_iter().any(|g|g.contains_player(game, player))
    }
    pub fn contains_player(&self, game: &Game, player: PlayerReference)->bool{
        let players: &VecSet<PlayerReference> = self.players(game);
        players.contains(&player)
    }
    pub fn in_same_group(game: &Game, a: PlayerReference, b: PlayerReference)->bool{
        InsiderGroupID::all().iter().any(|group| group.contains_player(game, b) && group.contains_player(game, a))
    }
    pub fn all_groups_with_player(game: &Game, player_ref: PlayerReference)->VecSet<InsiderGroupID>{
        InsiderGroupID::all()
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
            if Self::get_insider_group_from_chat_group(&chat_group).is_none() {continue};
            game.add_message_to_chat_group(chat_group, message.clone());
            message_sent = true;
        }
        if !message_sent && send_private_backup {
            player.add_private_chat_message(game, message);
        }
    }
}