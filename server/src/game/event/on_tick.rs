use crate::game::{
    components::ascend::Ascend, controllers::Controllers, Game
};

pub struct OnTick;

impl OnTick{
    pub fn new()->Self{
        Self{}
    }
    pub fn invoke(&self, game: &mut Game){
        Controllers::on_tick(game);
        Ascend::on_tick(game);
    }
}