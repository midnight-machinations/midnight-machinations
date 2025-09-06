
use crate::{
    game::{
        chat::{ChatMessage, ChatMessageVariant, ChatPlayerComponent},
        components::player_component::PlayerComponent, controllers::{ControllerID, IntegerSelection},
        event::{on_conceal_role::OnConcealRole, Event as _}, modifiers::ModifierID, verdict::Verdict, Game
    },
    packet::ToClientPacket,
    vec_set::VecSet
};

use super::PlayerReference;


impl PlayerReference{
    pub fn name<'a>(&self, game: &'a Game) -> &'a String {
        &self.deref(game).name
    }
    pub fn set_name(&self, game: &mut Game, new_name: String) {
        self.deref_mut(game).name = new_name;

        game.send_packet_to_all(ToClientPacket::GamePlayers { 
            players: PlayerReference::all_players(game).map(|p| p.name(game)).cloned().collect()
        });
    }

    pub fn alive(&self, game: &Game) -> bool{
        self.deref(game).alive
    }
    pub fn set_alive(&self, game: &mut Game, alive: bool){
        self.deref_mut(game).alive = alive;

        let mut alive_players = vec![];
        for player in PlayerReference::all_players(game){
            alive_players.push(player.deref(game).alive);
        }
        game.send_packet_to_all(ToClientPacket::PlayerAlive { alive: alive_players });
        game.count_nomination_and_start_trial(
            game.modifier_settings().is_enabled(ModifierID::UnscheduledNominations)
        );
    }
    
    pub fn notes<'a>(&self, game: &'a Game) -> &'a Vec<String> {
        &self.deref(game).notes
    }
    pub fn set_notes(&self, game: &mut Game, notes: Vec<String>){
        self.deref_mut(game).notes = notes;
        self.send_packet(game, ToClientPacket::YourNotes { notes: self.deref(game).notes.clone() })
    }

    pub fn crossed_out_outlines<'a>(&self, game: &'a Game) -> &'a Vec<u8> {
        &self.deref(game).crossed_out_outlines
    }
    pub fn set_crossed_out_outlines(&self, game: &mut Game, crossed_out_outlines: Vec<u8>){
        self.deref_mut(game).crossed_out_outlines = crossed_out_outlines;
        self.send_packet(game, ToClientPacket::YourCrossedOutOutlines { crossed_out_outlines: self.deref(game).crossed_out_outlines.clone() });
    }
    
    pub fn death_note<'a>(&self, game: &'a Game) -> &'a Option<String> {
        &self.deref(game).death_note
    }
    pub fn set_death_note(&self, game: &mut Game, death_note: Option<String>){
        self.deref_mut(game).death_note = death_note;
        self.send_packet(game, ToClientPacket::YourDeathNote { death_note: self.deref(game).death_note.clone() })
    }
    
    pub fn revealed_players<'a>(&self, game: &'a Game) -> &'a VecSet<PlayerReference>{
        &self.deref(game).role_labels
    }  
    pub fn reveal_players_role(&self, game: &mut Game, revealed_player: PlayerReference){
        if
            revealed_player != *self &&
            revealed_player.alive(game) &&
            self.deref_mut(game).role_labels.insert(revealed_player).is_none()
        {
            self.add_private_chat_message(game, ChatMessageVariant::PlayersRoleRevealed { player: revealed_player.index(), role: revealed_player.role(game) })
        }


        self.send_packet(game, ToClientPacket::YourRoleLabels{
            role_labels: PlayerReference::ref_vec_map_to_index(self.revealed_players_map(game)) 
        });
    }
    pub fn conceal_players_role(&self, game: &mut Game, concealed_player: PlayerReference){
        if self.deref_mut(game).role_labels.remove(&concealed_player).is_some() {
            self.add_private_chat_message(game, ChatMessageVariant::PlayersRoleConcealed { player: concealed_player.index() })
        }

        self.send_packet(game, ToClientPacket::YourRoleLabels{
            role_labels: PlayerReference::ref_vec_map_to_index(self.revealed_players_map(game)) 
        });

        OnConcealRole::new(*self, concealed_player).invoke(game);
    }

    pub fn add_private_chat_message(&self, game: &mut Game, message: ChatMessageVariant) {
        let message = ChatMessage::new_private(message);

        self.add_chat_message(game, message.clone());
    }
    pub fn add_private_chat_messages(&self, game: &mut Game, messages: Vec<ChatMessageVariant>){
        for message in messages {
            self.add_private_chat_message(game, message);
        }
    }
    pub fn add_chat_message(&self, game: &mut Game, message: ChatMessage) {
        PlayerComponent::<ChatPlayerComponent>::add_chat_message(game, *self, message);
    }
    pub fn chat_messages<'a>(&self, game: &'a Game) -> &'a Vec<ChatMessage> {
        PlayerComponent::<ChatPlayerComponent>::chat_messages(game, *self)
    }

    /* 
    Voting
    */
    pub fn verdict(&self, game: &Game) -> Verdict{
        match (ControllerID::Judge{player: *self}.get_integer_selection(game)) {
            Some(IntegerSelection(0)) => Verdict::Innocent,
            Some(IntegerSelection(1)) => Verdict::Guilty,
            _ => Verdict::Abstain
        }
    }
}



