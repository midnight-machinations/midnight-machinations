use crate::{game::prelude::*, packet::ToClientPacket};
pub mod grave;
pub mod grave_reference;

#[derive(Default)]
pub struct Graves{
    graves: Vec<Grave>,
}
impl Graves{
    pub fn add_grave(game: &mut Game, grave: Grave) {
        if let Ok(grave_index) = game.graves.graves.len().try_into() {
            game.graves.graves.push(grave.clone());

            if let Some(grave_ref) = GraveReference::new(game, grave_index) {
                OnGraveAdded::new(grave_ref).as_invokable().invoke(game);
            }
        }
    }
    pub fn on_grave_added(game: &mut Game, event: &OnGraveAdded, _fold: &mut (), _priority: ()){   
        let grave = event.grave.deref(game).clone();     
        game.send_packet_to_all(ToClientPacket::AddGrave{grave: grave.clone(), grave_ref: event.grave});
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PlayerDied { grave: grave.clone() });

        
        for other_player_ref in PlayerReference::all_players(game){
            other_player_ref.conceal_players_role(game, grave.player);
        }
    }
}