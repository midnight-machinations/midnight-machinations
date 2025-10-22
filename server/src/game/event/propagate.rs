use crate::game::{event::EventData, Game};

pub trait EventListener<E: EventData>{
    fn on_event(self, game: &mut Game, data: &E, fold: &mut E::FoldValue, priority: E::Priority);
}
impl<T, E, I> EventListener<E> for I where
    T: EventListener<E>,
    E: EventData,
    I: Iterator<Item = T>
{
    fn on_event(self, game: &mut Game, data: &E, fold: &mut <E as EventData>::FoldValue, priority: <E as EventData>::Priority) {
        self.for_each(|e|e.on_event(game, data, fold, priority));
    }
}