use crate::game::{
    controllers::Controllers, 
    Game
};

pub struct OnTick;

impl OnTick{
    pub fn new()->Self{
        Self{}
    }
    pub fn invoke(&self, game: &mut Game){
        Controllers::on_tick(game);
    }
}