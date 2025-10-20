use crate::game::{
    components::ascend::Ascend, controllers::Controllers, event::EventData,
};

pub struct OnTick;

impl OnTick{
    pub fn new()->(Self, ()){
        (Self{}, ())
    }
}
impl EventData for OnTick{
    type FoldValue = ();
    type Priority = ();

    fn listeners() -> Vec<super::EventListenerFunction<Self>> {vec![
        Controllers::on_tick,
        Ascend::on_tick
    ]}
}