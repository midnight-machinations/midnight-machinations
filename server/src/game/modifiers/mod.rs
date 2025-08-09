pub mod obscured_graves;
pub mod dead_can_chat;
pub mod abstaining;
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

use dead_can_chat::DeadCanChat;
use hidden_whispers::HiddenWhispers;
use abstaining::Abstaining;
use no_chat::NoChat;
use no_due_process::AutoGuilty;
use no_night_chat::NoNightChat;
use no_trial::NoTrialPhases;
use no_whispers::NoWhispers;
use obscured_graves::ObscuredGraves;
use no_death_cause::NoDeathCause;
use role_set_grave_killers::RoleSetGraveKillers;
use unscheduled_nominations::UnscheduledNominations;
use hidden_nomination_votes::HiddenNominationVotes;
use hidden_verdict_votes::HiddenVerdictVotes;
use forfeit_vote::ForfeitNominationVote;

use serde::{Deserialize, Serialize};
use skip_day_1::SkipDay1;
use two_thirds_majority::TwoThirdsMajority;

use crate::{vec_map::VecMap, vec_set::VecSet};

use super::{ability_input::AbilityInput,
    event::{
        on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority},
        on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    },
    grave::GraveReference, player::PlayerReference, Game
};


#[enum_delegate::register]
pub trait ModifierTrait where Self: Clone + Sized{
    fn on_ability_input_received(self, _game: &mut Game, _actor_ref: PlayerReference, _input: AbilityInput) {}
    fn on_midnight(self, _game: &mut Game, _priority: OnMidnightPriority) {}
    fn before_phase_end(self, _game: &mut Game, _phase: super::phase::PhaseType) {}
    fn on_phase_start(self, _game: &mut Game, _phase: super::phase::PhaseState) {}
    fn on_grave_added(self, _game: &mut Game, _event: GraveReference) {}
    fn on_game_start(self, _game: &mut Game) {}
    fn on_any_death(self, _game: &mut Game, _player: PlayerReference) {}
    fn before_initial_role_creation(self, _game: &mut Game) {}
    fn on_whisper(self, _game: &mut Game, _event: &OnWhisper, _fold: &mut WhisperFold, _priority: WhisperPriority) {}
}

#[enum_delegate::implement(ModifierTrait)]
#[derive(Clone, PartialEq, Eq)]
pub enum ModifierState{
    ObscuredGraves(ObscuredGraves),
    SkipDay1(SkipDay1),
    DeadCanChat(DeadCanChat),
    Abstaining(Abstaining),
    NoDeathCause(NoDeathCause),
    RoleSetGraveKillers(RoleSetGraveKillers),
    AutoGuilty(AutoGuilty),
    TwoThirdsMajority(TwoThirdsMajority),
    NoTrialPhases(NoTrialPhases),
    NoWhispers(NoWhispers),
    NoNightChat(NoNightChat),
    NoChat(NoChat),
    HiddenWhispers(HiddenWhispers),
    UnscheduledNominations(UnscheduledNominations),
    HiddenNominationVotes(HiddenNominationVotes),
    HiddenVerdictVotes(HiddenVerdictVotes),
    ForfeitNominationVote(ForfeitNominationVote),
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ModifierType{
    ObscuredGraves,
    SkipDay1,
    DeadCanChat,
    Abstaining,
    NoDeathCause,
    RoleSetGraveKillers,
    AutoGuilty,
    TwoThirdsMajority,
    NoTrialPhases,
    NoWhispers,
    NoNightChat,
    NoChat,
    HiddenWhispers,
    UnscheduledNominations,
    HiddenNominationVotes,
    HiddenVerdictVotes,
    ForfeitNominationVote,
}
impl ModifierType{
    pub fn default_state(&self)->ModifierState{
        match self{
            Self::ObscuredGraves => ModifierState::ObscuredGraves(ObscuredGraves),
            Self::SkipDay1 => ModifierState::SkipDay1(SkipDay1),
            Self::DeadCanChat => ModifierState::DeadCanChat(DeadCanChat),
            Self::Abstaining => ModifierState::Abstaining(Abstaining),
            Self::NoDeathCause => ModifierState::NoDeathCause(NoDeathCause),
            Self::RoleSetGraveKillers => ModifierState::RoleSetGraveKillers(RoleSetGraveKillers),
            Self::AutoGuilty => ModifierState::AutoGuilty(AutoGuilty),
            Self::TwoThirdsMajority => ModifierState::TwoThirdsMajority(TwoThirdsMajority),
            Self::NoTrialPhases => ModifierState::NoTrialPhases(NoTrialPhases),
            Self::NoWhispers => ModifierState::NoWhispers(NoWhispers),
            Self::NoNightChat => ModifierState::NoNightChat(NoNightChat),
            Self::NoChat => ModifierState::NoChat(NoChat),
            Self::HiddenWhispers => ModifierState::HiddenWhispers(HiddenWhispers),
            Self::UnscheduledNominations => ModifierState::UnscheduledNominations(UnscheduledNominations),
            Self::HiddenNominationVotes => ModifierState::HiddenNominationVotes(HiddenNominationVotes),
            Self::HiddenVerdictVotes => ModifierState::HiddenVerdictVotes(HiddenVerdictVotes),
            Self::ForfeitNominationVote => ModifierState::ForfeitNominationVote(ForfeitNominationVote),
        }
    }
}
impl From<&ModifierState> for ModifierType{
    fn from(state: &ModifierState)->Self{
        match state {
            ModifierState::ObscuredGraves(_) => Self::ObscuredGraves,
            ModifierState::SkipDay1(_) => Self::SkipDay1,
            ModifierState::DeadCanChat(_) => Self::DeadCanChat,
            ModifierState::Abstaining(_) => Self::Abstaining,
            ModifierState::NoDeathCause(_) => Self::NoDeathCause,
            ModifierState::RoleSetGraveKillers(_) => Self::RoleSetGraveKillers,
            ModifierState::AutoGuilty(_) => Self::AutoGuilty,
            ModifierState::TwoThirdsMajority(_) => Self::TwoThirdsMajority,
            ModifierState::NoTrialPhases(_) => Self::NoTrialPhases,
            ModifierState::NoWhispers(_) => Self::NoWhispers,
            ModifierState::NoNightChat(_) => Self::NoNightChat,
            ModifierState::NoChat(_) => Self::NoChat,
            ModifierState::HiddenWhispers(_) => Self::HiddenWhispers,
            ModifierState::UnscheduledNominations(_) => Self::UnscheduledNominations,
            ModifierState::HiddenNominationVotes(_) => Self::HiddenNominationVotes,
            ModifierState::HiddenVerdictVotes(_) => Self::HiddenVerdictVotes,
            ModifierState::ForfeitNominationVote(_) => Self::ForfeitNominationVote,
        }
    }
}




#[derive(Default)]
pub struct Modifiers{
    modifiers: VecMap<ModifierType, ModifierState>,
}

impl Modifiers{
    pub fn is_enabled(game: &Game, modifier: ModifierType)->bool{
        game.modifiers.modifiers.contains(&modifier)
    }
    pub fn get_modifier(game: &Game, modifier: ModifierType)->Option<&ModifierState>{
        game.modifiers.modifiers.get(&modifier)
    }
    pub fn get_modifier_inner<'a, T>(game: &'a Game, modifier: ModifierType)->Option<&'a T>
    where 
        // T: ModifierTrait,
        // ModifierState: TryInto<T>,
        &'a ModifierState: TryInto<&'a T>,
    {
        game.modifiers.modifiers.get(&modifier).and_then(|s|
            s.try_into().ok()
        )
    }
    pub fn set_modifier(game: &mut Game, state: ModifierState){
        game.modifiers.modifiers.insert(
            <&ModifierState as Into<ModifierType>>::into(&state).clone(),
            state
        );
    }
    pub fn default_from_settings(modifiers: VecSet<ModifierType>)->Self{
        let modifiers = modifiers
            .into_iter().map(|m|{let state = m.default_state(); (m, state)}).collect();
        Self{
            modifiers,
        }
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, _fold: &mut MidnightVariables, priority: OnMidnightPriority){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_midnight(game, priority);
        }
    }
    pub fn on_ability_input_received(game: &mut Game, actor_ref: crate::game::player::PlayerReference, input: crate::game::ability_input::AbilityInput){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_ability_input_received(game, actor_ref, input.clone());
        }
    }
    pub fn on_grave_added(game: &mut Game, event: GraveReference){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_grave_added(game, event);
        }
    }
    pub fn on_game_start(game: &mut Game){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_game_start(game);
        }
    }
    pub fn before_phase_end(game: &mut Game, phase: super::phase::PhaseType){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.before_phase_end(game, phase);
        }
    }
    pub fn on_phase_start(game: &mut Game, phase: super::phase::PhaseState){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_phase_start(game, phase.clone());
        }
    }
    pub fn on_any_death(game: &mut Game, player: crate::game::player::PlayerReference){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_any_death(game, player);
        }
    }
    pub fn before_initial_role_creation(game: &mut Game){
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.before_initial_role_creation(game);
        }
    }
    pub fn on_whisper(game: &mut Game, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        for modifier in game.modifiers.modifiers.clone(){
            modifier.1.on_whisper(game, event, fold, priority);
        }
    }
}