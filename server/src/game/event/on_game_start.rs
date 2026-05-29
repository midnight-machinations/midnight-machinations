use crate::game::event::EventData;
use super::{EventListenerFunction, LegacyEventData};

#[must_use = "Event must be invoked"]
pub struct OnGameStart;
impl OnGameStart{
    pub fn new()->(Self, ()){(Self, ())}
}
impl EventData for OnGameStart{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnGameStart{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::modifiers::ModifierSettings::on_game_start,
        crate::game::components::mafia::Mafia::on_game_start,
        crate::game::components::cult::Cult::on_game_start,
        crate::game::components::enfranchise::EnfranchiseComponent::on_game_start,
        crate::game::components::forfeit_vote::ForfeitNominationVote::on_game_start,
    ]}
}