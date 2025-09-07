pub mod obscured_graves;
pub mod dead_can_chat;
pub mod abstaining;
pub mod custom_role_limits;
pub mod no_death_cause;
pub mod role_set_grave_killers;
pub mod no_due_process;
pub mod two_thirds_majority;
pub mod no_trial;
pub mod no_whispers;
pub mod no_night_chat;
pub mod no_chat;
pub mod unscheduled_nominations;
pub mod skip_day_1;
pub mod hidden_whispers;
pub mod hidden_nomination_votes;
pub mod hidden_verdict_votes;
pub mod forfeit_vote;
pub mod random_player_names;

use crate::{game::{components::graves::grave_reference::GraveReference, event::{before_phase_end::BeforePhaseEnd, on_any_death::OnAnyDeath, on_phase_start::OnPhaseStart}}, vec_map::VecMap};

use serde::{Serialize, Deserialize};

use super::{controllers::ControllerInput,
    event::{
        on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority},
        on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    },
    player::PlayerReference, Game
};

pub trait ModifierStateImpl where Self: Clone + Sized + Default + Serialize + for<'de> Deserialize<'de>{
    fn on_ability_input_received(self, _game: &mut Game, _actor_ref: PlayerReference, _input: ControllerInput) {}
    fn on_midnight(self, _game: &mut Game, _priority: OnMidnightPriority) {}
    fn before_phase_end(self, _game: &mut Game, _phase: super::phase::PhaseType) {}
    fn on_phase_start(self, _game: &mut Game, _event: &OnPhaseStart, _fold: &mut (), _priority: ()) {}
    fn on_grave_added(self, _game: &mut Game, _event: GraveReference) {}
    fn on_game_start(self, _game: &mut Game) {}
    fn on_any_death(self, _game: &mut Game, _player: PlayerReference) {}
    fn on_whisper(self, _game: &mut Game, _event: &OnWhisper, _fold: &mut WhisperFold, _priority: WhisperPriority) {}
}

macros::modifiers! {
    obscured_graves: ObscuredGraves,
    skip_day_1: SkipDay1,
    dead_can_chat: DeadCanChat,
    abstaining: Abstaining,
    no_death_cause: NoDeathCause,
    role_set_grave_killers: RoleSetGraveKillers,
    no_due_process: AutoGuilty,
    two_thirds_majority: TwoThirdsMajority,
    no_trial: NoTrialPhases,
    no_whispers: NoWhispers,
    no_night_chat: NoNightChat,
    no_chat: NoChat,
    hidden_whispers: HiddenWhispers,
    unscheduled_nominations: UnscheduledNominations,
    hidden_nomination_votes: HiddenNominationVotes,
    hidden_verdict_votes: HiddenVerdictVotes,
    forfeit_vote: ForfeitNominationVote,
    random_player_names: RandomPlayerNames,
    custom_role_limits: CustomRoleLimits
}


#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ModifierSettings{
    modifiers: VecMap<ModifierID, ModifierState>,
}

impl ModifierSettings{
    pub fn is_enabled(&self, modifier: ModifierID)->bool{
        self.modifiers.contains(&modifier)
    }
    pub fn get_modifier(&self, modifier: ModifierID)->Option<&ModifierState>{
        self.modifiers.get(&modifier)
    }
    pub fn get_modifier_inner<'a, T>(&'a self, modifier: ModifierID)->Option<&'a T>
    where 
        // T: ModifierTrait,
        // ModifierState: TryInto<T>,
        &'a ModifierState: TryInto<&'a T>,
    {
        self.modifiers.get(&modifier).and_then(|s|
            s.try_into().ok()
        )
    }
    pub fn set_modifier(&mut self, state: ModifierState){
        self.modifiers.insert(
            <&ModifierState as Into<ModifierID>>::into(&state),
            state
        );
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, _fold: &mut MidnightVariables, priority: OnMidnightPriority){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_midnight(game, priority);
        }
    }
    pub fn on_ability_input_received(game: &mut Game, actor_ref: crate::game::player::PlayerReference, input: crate::game::controllers::ControllerInput){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_ability_input_received(game, actor_ref, input.clone());
        }
    }
    pub fn on_grave_added(game: &mut Game, event: GraveReference){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_grave_added(game, event);
        }
    }
    pub fn on_game_start(game: &mut Game){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_game_start(game);
        }
    }
    pub fn before_phase_end(game: &mut Game, event: &BeforePhaseEnd, _fold: &mut (), _priority: ()){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.before_phase_end(game, event.phase);
        }
    }
    pub fn on_phase_start(game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_phase_start(game, event, _fold, _priority);
        }
    }
    pub fn on_any_death(game: &mut Game, event: &OnAnyDeath, _fold: &mut (), _priority: ()){
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_any_death(game, event.dead_player);
        }
    }
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        for modifier in game.modifier_settings().modifiers.clone(){
            modifier.1.on_whisper(game, event, fold, priority);
        }
    }
}

mod macros {
    macro_rules! modifiers {
        ($($file:ident: $name:ident),*) => {
            #[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
            #[serde(tag = "type", rename_all = "camelCase")]
            pub enum ModifierState {
                $(
                    $name($file::$name),
                )*
            }

            #[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
            #[serde(rename_all = "camelCase")]
            pub enum ModifierID {
                $(
                    $name,
                )*
            }

            impl ModifierID {
                pub fn default_state(&self)->ModifierState{
                    match self{
                        $(
                            Self::$name => ModifierState::$name($file::$name::default()),
                        )*
                    }
                }
            }

            impl From<&ModifierState> for ModifierID {
                fn from(state: &ModifierState)->Self{
                    match state {
                        $(
                            ModifierState::$name(_) => Self::$name,
                        )*
                    }
                }
            }

            impl ModifierState {
                fn on_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input: ControllerInput) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_ability_input_received(game, actor_ref, input),
                        )*
                    }
                }
                fn on_midnight(self, game: &mut Game, priority: OnMidnightPriority) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_midnight(game, priority),
                        )*
                    }
                }
                fn before_phase_end(self, game: &mut Game, phase: super::phase::PhaseType) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.before_phase_end(game, phase),
                        )*
                    }
                }
                fn on_phase_start(self, game: &mut Game, event: &OnPhaseStart, _fold: &mut (), _priority: ()) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_phase_start(game, event, _fold, _priority),
                        )*
                    }
                }
                fn on_grave_added(self, game: &mut Game, event: GraveReference) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_grave_added(game, event),
                        )*
                    }
                }
                fn on_game_start(self, game: &mut Game) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_game_start(game),
                        )*
                    }
                }
                fn on_any_death(self, game: &mut Game, player: PlayerReference) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_any_death(game, player),
                        )*
                    }
                }
                fn on_whisper(self, game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
                    match self {
                        $(
                            ModifierState::$name(s) => s.on_whisper(game, event, fold, priority),
                        )*
                    }
                }
            }
        }
    }

    pub(super) use modifiers;
}