use crate::{game::{chat::ChatMessageVariant, player::PlayerReference, Game}, packet::ToClientPacket, vec_map::VecMap, vec_set::VecSet};
use serde::{Deserialize, Serialize};
use vec1::vec1;


#[derive(Default)]
pub struct Tags{
    tags: VecMap<TagSetID, TagsSet>
}
impl Tags{
    pub fn add_tag(game: &mut Game, id: TagSetID, tagged_player: PlayerReference){
        let added = if let Some(val) = game.tags.tags.get_mut(&id){
            val.insert_tag(tagged_player).is_none()
        }else{
            let mut new_set = TagsSet::new();
            new_set.insert_tag(tagged_player);
            game.tags.tags.insert(id.clone(),  new_set);
            true
        };

        if added {
            if let Some(tags_set) = game.tags.tags.get(&id){
                Self::send_to_clients(game, tags_set.viewers());
                for player in tags_set.viewers().clone() {
                    player.add_private_chat_message(game, ChatMessageVariant::TagAdded { player: tagged_player, tag: id.get_tag() });
                }
            }
        }
    }
    pub fn remove_tag(game: &mut Game, id: TagSetID, tagged_player: PlayerReference){
        let removed = if let Some(val) = game.tags.tags.get_mut(&id){
            val.remove_tag(tagged_player).is_some()
        }else{
            false
        };

        if removed {
            if let Some(tags_set) = game.tags.tags.get(&id){
                Self::send_to_clients(game, tags_set.viewers());
                for player in tags_set.viewers().clone() {
                    player.add_private_chat_message(game, ChatMessageVariant::TagRemoved { player: tagged_player, tag: id.get_tag() });
                }
            }
        }
    }
    
    pub fn add_viewer(game: &mut Game, id: TagSetID, player: PlayerReference){
        let added = if let Some(val) = game.tags.tags.get_mut(&id){
            val.insert_viewer(player).is_none()
        }else{
            let mut new_set = TagsSet::new();
            new_set.insert_viewer(player);
            game.tags.tags.insert(id.clone(),  new_set);
            true
        };

        if added {
            if let Some(tags_set) = game.tags.tags.get(&id){
                Self::send_to_client(game, player);
                for tagged_player in tags_set.tagged().clone() {
                    player.add_private_chat_message(game, ChatMessageVariant::TagAdded { player: tagged_player, tag: id.get_tag() });
                }
            }
        }
    }
    pub fn remove_viewer(game: &mut Game, id: TagSetID, player: PlayerReference){
        let removed = if let Some(val) = game.tags.tags.get_mut(&id){
            val.remove_viewer(player).is_some()
        }else{
            false
        };

        if removed {
            if let Some(tags_set) = game.tags.tags.get(&id){
                Self::send_to_client(game, player);
                for tagged_player in tags_set.tagged().clone() {
                    player.add_private_chat_message(game, ChatMessageVariant::TagRemoved { player: tagged_player, tag: id.get_tag() });
                }
            }
        }
    }

    pub fn set_tagged(game: &mut Game, id: TagSetID, tagged_players: &VecSet<PlayerReference>){
        for player in PlayerReference::all_players(game) {
            if tagged_players.contains(&player) {
                Self::add_tag(game, id.clone(), player);
            }else {
                Self::remove_tag(game, id.clone(), player);
            }
        }
    }
    pub fn set_viewers(game: &mut Game, id: TagSetID, viewers: &VecSet<PlayerReference>){
        for player in PlayerReference::all_players(game) {
            if viewers.contains(&player) {
                Self::add_viewer(game, id.clone(), player);
            }else {
                Self::remove_viewer(game, id.clone(), player);
            }
        }
    }

    pub fn has_tag(game: &Game, id: TagSetID, player: PlayerReference)->bool{
        game.tags.tags.get(&id).is_some_and(|set|set.tagged().contains(&player))
    }
    pub fn tagged(game: &Game, id: TagSetID)->VecSet<PlayerReference>{
        if let Some(tags) = game.tags.tags.get(&id) {tags.tagged().clone()} else {VecSet::new()}
    }




    pub fn send_to_clients(game: &Game, players: &VecSet<PlayerReference>){
        for player in players.iter(){
            Self::send_to_client(game, *player);
        }
    }
    pub fn send_to_client(game: &Game, player: PlayerReference){
        player.send_packet(game, ToClientPacket::YourPlayerTags { player_tags: Self::player_tags_map(game, player) });
    }
    fn player_tags_map(game: &Game, player: PlayerReference)->VecMap<PlayerReference, vec1::Vec1<Tag>>{
        let mut player_tags: VecMap<PlayerReference, vec1::Vec1<Tag>> = VecMap::new();
        for (id, tags_set) in game.tags.tags.iter(){
            if !tags_set.viewers().contains(&player) {continue}

            for tagged_player in tags_set.tagged().iter(){

                if let Some(tags) = player_tags.get_mut(tagged_player){
                    tags.push(id.get_tag());
                }else{
                    player_tags.insert(*tagged_player, vec1!(id.get_tag()));
                }

            }
        }
        player_tags
    }
}
struct TagsSet{
    viewers: VecSet<PlayerReference>,
    tagged: VecSet<PlayerReference>
}
impl TagsSet{
    fn new()->Self{
        Self { viewers: VecSet::new(), tagged: VecSet::new() }
    }
    fn insert_tag(&mut self, player: PlayerReference)->Option<PlayerReference>{
        self.tagged.insert(player)
    }
    fn remove_tag(&mut self, player: PlayerReference)->Option<PlayerReference>{
        self.tagged.remove(&player)
    }
    fn insert_viewer(&mut self, player: PlayerReference)->Option<PlayerReference>{
        self.viewers.insert(player)
    }
    fn remove_viewer(&mut self, player: PlayerReference)->Option<PlayerReference>{
        self.viewers.remove(&player)
    }

    fn viewers(&self)->&VecSet<PlayerReference>{
        &self.viewers
    }
    fn tagged(&self)->&VecSet<PlayerReference>{
        &self.tagged
    }
}
#[derive(PartialEq, Eq, Clone)]
pub enum TagSetID{
    ArsonistDoused,
    MorticianTag(PlayerReference),
    Framer(PlayerReference),
    Enfranchised,
    PuppeteerMarionette,
    SyndicateRecruit,
    SyndicateBackup,
    SyndicateGun,
    WerewolfTracked(PlayerReference),
    RevolutionaryTarget(PlayerReference),
    UzumakiSpiral(PlayerReference),
    ForfeitNominationVote
}
impl TagSetID{
    fn get_tag(&self)->Tag{
        match self {
            TagSetID::ArsonistDoused => Tag::Doused,
            TagSetID::MorticianTag(_) => Tag::MorticianTagged,
            TagSetID::Framer(_) => Tag::Frame,
            TagSetID::Enfranchised => Tag::Enfranchised,
            TagSetID::PuppeteerMarionette => Tag::PuppeteerMarionette,
            TagSetID::SyndicateRecruit => Tag::PuppeteerMarionette,
            TagSetID::SyndicateBackup => Tag::GodfatherBackup,
            TagSetID::SyndicateGun => Tag::SyndicateGun,
            TagSetID::WerewolfTracked(_) => Tag::WerewolfTracked,
            TagSetID::RevolutionaryTarget(_) => Tag::RevolutionaryTarget,
            TagSetID::UzumakiSpiral(_) => Tag::Spiraling,
            TagSetID::ForfeitNominationVote => Tag::ForfeitNominationVote,
        }
    }
}


#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Tag{
    Doused,
    MorticianTagged,
    Frame,
    Enfranchised,
    PuppeteerMarionette,
    GodfatherBackup,
    SyndicateGun,
    WerewolfTracked,
    RevolutionaryTarget,
    Spiraling,
    ForfeitNominationVote,
}