use std::borrow::Borrow;

use crate::game::{event::EventData, Game};

pub trait EventListener<E: EventData>{
    fn on_event(&self, _game: &mut Game, _data: &E, _fold: &mut E::FoldValue, _priority: E::Priority) {}
}
impl<B, E> EventListener<E> for [B] where
    B: Borrow<dyn EventListener<E>>,
    E: EventData,
{
    fn on_event(&self, game: &mut Game, data: &E, fold: &mut <E as EventData>::FoldValue, priority: <E as EventData>::Priority) {
        self.iter().for_each(|e| e.borrow().on_event(game, data, fold, priority));
    }
}