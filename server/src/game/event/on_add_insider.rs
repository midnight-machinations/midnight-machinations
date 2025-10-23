use crate::game::{
    abilities_component::Abilities, components::{
        insider_group::InsiderGroupID, mafia::Mafia,
        mafia_recruits::MafiaRecruits, puppeteer_marionette::PuppeteerMarionette
    }, player::PlayerReference
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
            Abilities::on_event,
            PuppeteerMarionette::on_add_insider,
            Mafia::on_add_insider,
            MafiaRecruits::on_add_insider,
        ]
    }
}