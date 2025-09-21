use std::{any::Any, marker::PhantomData};

use crate::game::Game;

pub struct EventHandlers {
    handlers: Vec<Box<dyn EventHandlerTrait>>,
}

impl EventHandlers {
    pub fn new()->Self{
        Self { handlers: Vec::default() }
    }
    pub fn register<L, E, S, const P: usize>(game: &mut Game, listener: L)
        where 
        L: EventListener<E, S, P> + Send + Clone + EventListenerStateProvider<S> + 'static,
        E: Event + Send + 'static,
        S: Send + 'static, 
    {
        game.event_handlers.handlers.push(
            Box::new(
                EventHandler::<L, E, S, P> {
                    listener,
                    _marker: PhantomData,
                }
            )
        );
    }
    pub fn unregister(game: &mut Game){
        let mut handlers = game.event_handlers.take_handlers();
        handlers.retain(|h|h.should_unregister(game));
        Self::return_handlers(game, handlers);
    }
    
    /// When this is implemented into MM, you wouldnt need the &mut self parameter. you can just get &mut game.event_handlets
    fn invoke<E: Event + 'static>(game: &mut Game, event: E) {

        let mut handlers = game.event_handlers.take_handlers();

        let mut fold = event.new_fold();
        for p in E::Priority::into_iter(){
            for h in handlers.iter() {
                h.call(game, &event, &mut fold, &p);
            }
        }

        handlers.retain(|h|h.should_unregister(game));

        Self::return_handlers(game, handlers);
    }

    fn take_handlers(&mut self)->Vec<Box<dyn EventHandlerTrait>>{
        std::mem::take(&mut self.handlers)
    }
    fn return_handlers(game: &mut Game, handlers: Vec<Box<dyn EventHandlerTrait>>){
        game.event_handlers.handlers = handlers
    }
}

struct EventHandler<L, E, S, const P: usize = 0> {
    listener: L,
    _marker: PhantomData<(E, S)>,
}

trait EventHandlerTrait: Send {
    fn call(&self, game: &mut Game, event: &dyn Any, fold: &mut dyn Any, priority: &dyn Any);
    fn should_unregister(&self, game: &Game)->bool;
}
impl<L, E, S, const P: usize> EventHandlerTrait for EventHandler<L, E, S, P>
where
    E: Event + Send + 'static,
    E::Fold: 'static,
    L: EventListener<E, S, P> + Clone + Send + EventListenerStateProvider<S> + 'static,
    S: Send + 'static
{
    fn should_unregister(&self, game: &Game)->bool {self.listener.should_unregister(game)}
    fn call(&self, game: &mut Game, event: &dyn Any, fold: &mut dyn Any, priority: &dyn Any) {
        let Some(event) = event.downcast_ref::<E>() else { return; };
        let Some(fold) = fold.downcast_mut::<E::Fold>() else { return; };
        let Some(priority) = priority.downcast_ref::<E::Priority>() else { return; };

        if P != 0 && priority.into_usize() != P {return;}
        if self.should_unregister(game) {return;}
        
        let mut param: EventListenerParameters<'_, E> = EventListenerParameters { event, fold, priority };
        if let Some(state) = self.listener.get_state(game) {
            L::call(&state, game, &mut param);
        }
    }
}


// listener

/// P is the priority this event should listen to. 0 is a sentinel value for "listen to all priorities"
/// All priorities need to implement into_usize(), so those into_usize() should not ever return 0 (unless the priority is equal to ())
/// 
/// S is the state to fetch from game, not required, but effects should_unregister
pub trait EventListener<E: Event, S = (), const P: usize = 0> {
    fn should_unregister(&self, game: &Game)->bool {self.get_state(game).is_none()}
    fn call(state: &S, game: &mut Game, param: &mut EventListenerParameters<E>);
}
pub trait EventListenerStateProvider<S> {
    fn get_state(&self, _game: &Game) -> Option<S> {None}
}
impl<T> EventListenerStateProvider<()> for T {
    fn get_state(&self, _game: &Game) -> Option<()> { Some(()) }
}

pub struct EventListenerParameters<'a, E>
    where E: Event
{
    event: &'a E,
    fold: &'a mut E::Fold,
    priority: &'a E::Priority,
}

//event
pub trait Event {
    type Fold;
    type Priority: Priority;
    fn new_fold(&self)->Self::Fold;
    fn invoke(self, game: &mut Game)
    where
        Self: Sized + 'static,
    {
        EventHandlers::invoke(game, self);
    }
}

pub trait Priority: Sized + Copy {
    fn into_iter() -> impl Iterator<Item = Self>;
    fn into_usize(self) -> usize;
}
impl Priority for () {
    fn into_iter() -> impl Iterator<Item = Self> {
        std::iter::once(())
    }
    fn into_usize(self) -> usize {0}
}