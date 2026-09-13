use crate::game::{
    abilities_component::Abilities, components::{
        insider_group::InsiderGroupID, mafia::Mafia,
        mafia_recruits::MafiaRecruits, puppeteer_marionette::PuppeteerMarionette
    }, player::PlayerReference, Game
};
use super::EventData;

#[derive(Clone)]
pub struct OnAddInsider {
    pub player: PlayerReference,
    pub group: InsiderGroupID
}

impl OnAddInsider {
    pub fn new(player: PlayerReference, group: InsiderGroupID) -> (Self, ()) {
        (Self {
            player,
            group
        }, ())
    }
}

impl EventData for OnAddInsider {
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

    fn log(&self, _game: &Game, _fold: &()) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "player": self.player, "group": self.group }))
    }
}