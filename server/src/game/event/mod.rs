use super::Game;
pub(super) mod on_any_death;
pub(super) mod on_game_ending;
pub(super) mod on_phase_start;
pub(super) mod on_grave_added;
pub(super) mod on_game_start;
pub(super) mod on_role_switch;
pub(super) mod on_convert;
pub(super) mod on_ability_deletion;
pub(super) mod before_phase_end;
pub(super) mod on_midnight;
pub(super) mod on_conceal_role;
pub(super) mod on_controller_input_received;
pub(super) mod on_validated_ability_input_received;
pub(super) mod on_controller_selection_changed;
pub(super) mod on_tick;
pub(super) mod on_player_roleblocked;
pub(super) mod on_visit_wardblocked;
pub(super) mod on_whisper;
pub(super) mod on_add_insider;
pub(super) mod on_remove_insider;
pub(super) mod on_controller_changed;
pub(super) mod on_ability_creation;
pub(super) mod on_ability_edit;
pub(super) mod on_player_possessed;

pub mod prelude;
pub mod propagate;

pub trait EventPriority: Sized + Copy {
    fn values() -> Vec<Self>;
}

///
/// 
/// 
/// // Event listener type
/// // pub type EventListenerFunction<E: Event> = fn(&mut Game, &E, &mut E::FoldValue, E::Priority);
/// 
pub trait EventData: Sized {
    type FoldValue;
    type Priority: EventPriority;

    fn listeners() -> Vec<EventListenerFunction<Self>>;
    fn on_event(_game: &mut Game, _data: &Self, _fold: &mut Self::FoldValue, _priority: Self::Priority) {}
}
pub trait Invokable{
    fn invoke(self, game: &mut Game)->Self;
}
impl<E: EventData> Invokable for (&E, &mut E::FoldValue) {
    fn invoke(self, game: &mut Game) -> Self {
        for priority in E::Priority::values() {
            for listener in E::listeners() {
                listener(game, self.0, self.1, priority);
            }
            E::on_event(game, self.0, self.1, priority);
        }
        self
    }
}


#[expect(type_alias_bounds, reason="This is fine")]
pub type EventListenerFunction<E: EventData> = fn(&mut Game, &E, &mut E::FoldValue, E::Priority);

impl EventPriority for () {
    fn values() -> Vec<Self> {vec![()]}
}




#[macro_export]
macro_rules! event_priority {
    (
        $name:ident{
            $($variant:ident),*
        }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum $name {
            $($variant),*
        }
        impl $crate::game::event::EventPriority for $name {
            fn values() -> Vec<Self> {
                vec![$(Self::$variant),*]
            }
        }
    };
}




pub trait AsInvokable<E: EventData>{
    fn as_invokable(&mut self) -> (&E, &mut E::FoldValue);
}
impl<E: EventData> AsInvokable<E> for (E, E::FoldValue) {
    fn as_invokable(&mut self) -> (&E, &mut E::FoldValue) {
        let (ref e, ref mut f) = *self;
        (e, f)
    }
}