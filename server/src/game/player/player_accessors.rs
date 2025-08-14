use crate::{
    game::{
        chat::{ChatMessage, ChatMessageVariant}, event::{
            on_conceal_role::OnConcealRole, on_fast_forward::OnFastForward
        }, modifiers::{ModifierType, Modifiers}, role::{Role, RoleState}, verdict::Verdict, Game
    }, 
    packet::ToClientPacket, vec_set::VecSet, 
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
    
    pub fn role(&self, game: &Game) -> Role {
        self.deref(game).role_state.role()
    }
    pub fn role_state<'a>(&self, game: &'a Game) -> &'a RoleState {
        &self.deref(game).role_state
    }
    pub fn set_role_state(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        self.deref_mut(game).role_state = new_role_data.into();
        self.send_packet(game, ToClientPacket::YourRoleState {
            role_state: self.deref(game).role_state.clone().get_client_role_state(game, *self)
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
            Modifiers::is_enabled(game, ModifierType::UnscheduledNominations)
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
        self.deref_mut(game).chat_messages.push(message.clone());
        self.deref_mut(game).queued_chat_messages.push(message);
    }
    pub fn chat_messages<'a>(&self, game: &'a Game) -> &'a Vec<ChatMessage> {
        &self.deref(game).chat_messages
    }

    pub fn set_fast_forward_vote(&self, game: &mut Game, fast_forward_vote: bool) {
        self.deref_mut(game).fast_forward_vote = fast_forward_vote;

        self.send_packet(game, ToClientPacket::YourVoteFastForwardPhase { fast_forward: fast_forward_vote });

        if fast_forward_vote && !game.phase_machine.time_remaining.is_some_and(|d|d.is_zero()) && PlayerReference::all_players(game)
            .filter(|p|p.alive(game)&&(p.could_reconnect(game)||p.is_connected(game)))
            .all(|p| p.fast_forward_vote(game))
        {
            OnFastForward::invoke(game);
        }
    }
    pub fn fast_forward_vote(&self, game: &Game) -> bool{
        self.deref(game).fast_forward_vote
    }

    /* 
    Voting
    */
    pub fn verdict(&self, game: &Game) -> Verdict{
        self.deref(game).voting_variables.verdict
    }
    pub fn set_verdict(&self, game: &mut Game, mut verdict: Verdict){
        if verdict == Verdict::Abstain && !Modifiers::is_enabled(game, ModifierType::Abstaining) {
            verdict = Verdict::Innocent;
        }
        self.send_packet(game, ToClientPacket::YourJudgement { verdict });
        self.deref_mut(game).voting_variables.verdict = verdict;
    }
}



