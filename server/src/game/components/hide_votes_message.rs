use crate::game::{
    Game, chat::{ChatGroup, ChatMessageVariant}, event::on_tick::OnTick, modifiers::{hidden_nomination_votes::HiddenNominationVotes, hidden_verdict_votes::HiddenVerdictVotes}
};

#[derive(Default)]
pub struct HideVotesMessage{
    currently_hidden: bool
}

impl HideVotesMessage{
    pub fn update_hidden_votes(game: &mut Game) {
        let should_be_hidden =
            HiddenNominationVotes::nomination_votes_are_hidden(game) ||
            HiddenVerdictVotes::verdict_votes_are_hidden(game);
            
        if game.hide_votes_message.currently_hidden != should_be_hidden {
            game.hide_votes_message.currently_hidden = should_be_hidden;
            game.add_message_to_chat_group(
                ChatGroup::All,
                ChatMessageVariant::VotesHidden{value: should_be_hidden}
            );
            game.send_player_votes();
        }
    }
    pub fn on_tick(game: &mut Game, _event: &OnTick, _fold: &mut (), _priority: ()) {
        HideVotesMessage::update_hidden_votes(game)
    }
}