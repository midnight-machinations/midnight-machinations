use crate::game::event_handlers::Event;

pub struct OnTick;

impl OnTick{
    pub fn new()->Self{
        Self{}
    }
}
impl Event for OnTick{
    type Fold = ();
    type Priority = ();
    fn new_fold(&self)->Self::Fold {}
}