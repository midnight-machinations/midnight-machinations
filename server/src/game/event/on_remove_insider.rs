use crate::game::{
    abilities_component::Abilities, components::{
        cult::Cult, insider_group::InsiderGroupID, mafia::Mafia, mafia_recruits::MafiaRecruits, puppeteer_marionette::PuppeteerMarionette
    }, player::PlayerReference,
};
use super::EventData;

#[derive(Clone)]
pub struct OnRemoveInsider {
    pub player: PlayerReference,
    pub group: InsiderGroupID,
}

impl OnRemoveInsider {
    pub fn new(player: PlayerReference, group: InsiderGroupID) -> (Self, ()) {
        (
            Self {
                player,
                group,
            },
            ()
        )
    }
}

impl EventData for OnRemoveInsider {
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {
        vec![
            Abilities::on_event,
            PuppeteerMarionette::on_remove_insider,
            Cult::on_remove_insider,
            Mafia::on_remove_insider,
            MafiaRecruits::on_remove_insider,
        ]
    }
}