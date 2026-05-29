use crate::game::event::EventData;
use super::{EventListenerFunction, LegacyEventData};

pub struct OnTick;

impl OnTick{
    pub fn new()->(Self, ()){
        (Self{}, ())
    }
}
impl EventData for OnTick{
    type FoldValue = ();
}
#[allow(deprecated)]
impl LegacyEventData for OnTick{
    type FoldValue = ();
    type Priority = ();
    fn listeners() -> Vec<EventListenerFunction<Self>> {vec![
        crate::game::controllers::Controllers::on_tick,
        crate::game::components::ascend::Ascend::on_tick
    ]}
}