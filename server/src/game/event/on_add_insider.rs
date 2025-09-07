use crate::game::{
    abilities_component::Abilities, components::{
        insider_group::InsiderGroupID, mafia::Mafia,
        mafia_recruits::MafiaRecruits, puppeteer_marionette::PuppeteerMarionette
    }, player::PlayerReference, Game
};
use super::Event;

#[derive(Clone)]
pub struct OnAddInsider {
    pub player: PlayerReference,
    pub group: InsiderGroupID
}

impl OnAddInsider {
    pub fn new(player: PlayerReference, group: InsiderGroupID) -> Self {
        Self {
            player,
            group
        }
    }
}

impl Event for OnAddInsider {
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Abilities::on_add_insider,
            PuppeteerMarionette::on_add_insider,
            Mafia::on_add_insider,
            MafiaRecruits::on_add_insider,
        ]
    }
    
    fn initial_fold_value(&self, _game: &Game) -> Self::FoldValue {}
}