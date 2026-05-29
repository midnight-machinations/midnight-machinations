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

// ===== NEW EVENT SYSTEM =====

/// Trait for event data types. Each event defines its associated fold value type.
pub trait EventData: Sized {
    type FoldValue;
}

/// Trait for types that can listen to events. Implementations go on identifier types
/// like `PlayerReference`, `AbilityID`, etc.
pub trait EventListener<E> 
where
    E: EventData,
{
    fn on_event(&self, game: &mut Game, event: &E, fold: &mut E::FoldValue);
}

// ===== LEGACY COMPATIBILITY (deprecated) =====

#[allow(deprecated)]
pub trait EventPriority: Sized + Copy {
    fn values() -> Vec<Self>;
}

#[expect(type_alias_bounds, reason="Legacy compatibility")]
#[allow(deprecated)]
pub type EventListenerFunction<E: EventData> = fn(&mut Game, &E, &mut E::FoldValue, E::Priority);

#[allow(deprecated)]
pub trait LegacyEventData: Sized {
    type FoldValue;
    type Priority: EventPriority;
    fn listeners() -> Vec<EventListenerFunction<Self>>;
}

/// Blanket impl allows old events to work via `as_invokable().invoke()`
#[allow(deprecated)]
pub trait AsInvokable<E: LegacyEventData> {
    fn as_invokable(&mut self) -> (&E, &mut E::FoldValue);
}

#[allow(deprecated)]
pub trait Invokable {
    fn invoke(self, game: &mut Game) -> Self;
}

#[allow(deprecated)]
impl<E: LegacyEventData> AsInvokable<E> for (E, E::FoldValue) {
    fn as_invokable(&mut self) -> (&E, &mut E::FoldValue) {
        let (ref e, ref mut f) = *self;
        (e, f)
    }
}

#[allow(deprecated)]
impl<E: LegacyEventData> Invokable for (&E, &mut E::FoldValue) {
    fn invoke(self, game: &mut Game) -> Self {
        for priority in E::Priority::values() {
            for listener in E::listeners() {
                listener(game, self.0, self.1, priority);
            }
        }
        self
    }
}

impl EventPriority for () {
    fn values() -> Vec<Self> {vec![()]}
}

#[macro_export]
macro_rules! event_priority {
    (
        $name:ident{
            $($variant:ident = $value:expr),* $(,)?
        }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(u8)]
        pub enum $name {
            $($variant = $value),*
        }
        impl $name {
            pub const VALUES: &[$name] = &[$($name::$variant),*];
        }
        #[allow(deprecated)]
        impl $crate::game::event::EventPriority for $name {
            fn values() -> Vec<Self> {
                vec![$(Self::$variant),*]
            }
        }
    };
}