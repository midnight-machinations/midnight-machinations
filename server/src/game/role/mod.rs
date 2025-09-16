#![allow(clippy::single_match, reason = "May add more cases for more priorities later")]

use crate::game::components::graves::grave_reference::GraveReference;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority};
use crate::game::role_list_generation::criteria::GenerationCriterion;
use crate::vec_set::{vec_set, VecSet};
use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::Settings;
use crate::game::ModifierID;
use crate::game::modifiers::ModifierState;
use crate::game::chat::PlayerChatGroupMap;
use crate::game::phase::PhaseType;
use crate::game::attack_power::DefensePower;

use serde::{Serialize, Deserialize};

use super::components::win_condition::WinCondition;
use super::{
    controllers::*, components::insider_group::InsiderGroupID,
    event::{
        on_midnight::{MidnightVariables, OnMidnightPriority},
        on_whisper::{OnWhisper, WhisperFold, WhisperPriority}
    },
};

pub trait GetClientAbilityState<CRS> {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> CRS;
}
//Automatically implement this for the case where RoleState = ClientRoleState
impl<T: RoleStateTrait> GetClientAbilityState<T> for T {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> T {self}
}

pub trait RoleStateTrait: Clone + std::fmt::Debug + Default + GetClientAbilityState<<Self as RoleStateTrait>::ClientAbilityState> {
    type ClientAbilityState: Clone + std::fmt::Debug + Serialize;
    fn on_midnight(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _priority: OnMidnightPriority) {}

    fn controller_parameters_map(self, _game: &Game, _actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::default()
    }
    fn on_controller_selection_changed(self, _game: &mut Game, _actor_ref: PlayerReference, _id: ControllerID) {}
    fn on_validated_ability_input_received(self, _game: &mut Game, _actor_ref: PlayerReference, _input_player: PlayerReference, _ability_input: ControllerInput) {}

    fn convert_selection_to_visits(self, _game: &Game, _actor_ref: PlayerReference) -> Vec<Visit> {
        vec![]
    }

    fn send_player_chat_group_map(self, _game: &Game, _actor_ref: PlayerReference) -> PlayerChatGroupMap {
        PlayerChatGroupMap::new()
    }
    fn receive_player_chat_group_map(self, _game: &Game, _actor_ref: PlayerReference) -> PlayerChatGroupMap {
        PlayerChatGroupMap::new()
    }
    fn new_state(_game: &Game) -> Self {
        Self::default()
    }
    fn default_revealed_groups(self) -> VecSet<InsiderGroupID> {
        VecSet::new()
    }
    fn default_win_condition(self) -> WinCondition where RoleState: From<Self>{
        let role_state: RoleState = self.into();
        crate::game::role::common_role::default_win_condition(role_state.role())
    }

    fn on_phase_start(self, _game: &mut Game, _actor_ref: PlayerReference, _phase: PhaseType) {}
    fn on_ability_creation(self, _game: &mut Game, _actor_ref: PlayerReference, _event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, _priority: OnAbilityCreationPriority) {}
    fn on_ability_deletion(self, _game: &mut Game, _actor_ref: PlayerReference, _event: &OnAbilityDeletion, _fold: &mut (), _priority: OnAbilityDeletionPriority) {}
    fn on_role_switch(self, _game: &mut Game, _actor_ref: PlayerReference, _player: PlayerReference, _new: RoleState, _old: RoleState) {}
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {}
    fn on_grave_added(self, _game: &mut Game, _actor_ref: PlayerReference, _grave: GraveReference) {}
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn on_game_start(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn on_conceal_role(self, _game: &mut Game, _actor_ref: PlayerReference, _player: PlayerReference, _concealed_player: PlayerReference) {}
    fn on_player_roleblocked(self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, _invisible: bool) {
        common_role::on_player_roleblocked(midnight_variables, actor_ref, player);
    }
    fn on_visit_wardblocked(self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
        common_role::on_visit_wardblocked(midnight_variables, actor_ref, visit);
    }
    fn on_whisper(self, _game: &mut Game, _actor_ref: PlayerReference, _event: &OnWhisper, _fold: &mut WhisperFold, _priority: WhisperPriority) {}

    fn role_list_generation_criteria() -> Vec<GenerationCriterion> {
        vec![]
    }
}

// Creates the Role enum
macros::roles! {
    Jailor : jailor,
    Villager : villager,
    Drunk : drunk,

    Detective : detective,
    Lookout : lookout,
    Spy : spy,
    Tracker : tracker,
    Philosopher : philosopher,
    Psychic : psychic,
    Auditor : auditor,
    Snoop : snoop,
    Gossip : gossip,
    TallyClerk : tally_clerk,

    Doctor : doctor,
    Bodyguard : bodyguard,
    Cop : cop,
    Bouncer : bouncer,
    Engineer : engineer,
    Armorsmith : armorsmith,
    Steward : steward,

    Vigilante : vigilante,
    Veteran : veteran,
    Marksman: marksman,
    Deputy : deputy,
    Rabblerouser : rabblerouser,

    Escort : escort,
    Medium : medium,
    Retributionist : retributionist,
    Reporter : reporter,
    Mayor : mayor,
    Transporter : transporter,
    Porter : porter,
    Polymath : polymath,
    Courtesan : courtesan,

    // Mafia
    Godfather : godfather,
    Counterfeiter : counterfeiter,
    Impostor : impostor,
    Recruiter : recruiter,
    Mafioso : mafioso,
    MafiaKillingWildcard : mafia_killing_wildcard,

    Goon : made_man,
    Consort : consort,
    
    Hypnotist : hypnotist,
    Blackmailer : blackmailer,
    Cerenovous : cerenovous,
    Informant: informant,
    Necromancer : necromancer,
    Mortician : mortician,
    Framer : framer,
    Disguiser : disguiser,
    Forger : forger,
    Reeducator : reeducator,
    Ambusher : ambusher,
    MafiaSupportWildcard: mafia_support_wildcard,

    // Neutral
    Jester : jester,
    Revolutionary : revolutionary,
    Politician : politician,
    Doomsayer : doomsayer,
    Mercenary : mercenary,
    Wildcard : wild_card,
    TrueWildcard : true_wildcard,
    Martyr : martyr,
    SantaClaus : santa_claus,
    Krampus : krampus,
    Chronokaiser : chronokaiser,

    Witch : witch,
    Scarecrow : scarecrow,
    Warper : warper,
    Kidnapper : kidnapper,
    Pawn : pawn,
    Tailor : tailor,

    Arsonist : arsonist,
    Werewolf : werewolf,
    Ojo : ojo,
    Puppeteer: puppeteer,
    Pyrolisk : pyrolisk,
    Spiral : spiral,
    Kira : kira,
    Warden : warden,
    Yer : yer,
    FiendsWildcard : fiends_wildcard,
    SerialKiller : serial_killer,

    Apostle : apostle,
    Disciple : disciple,
    Zealot : zealot
}

pub(crate) mod common_role;

mod macros {
    macro_rules! roles {
        (
            $($name:ident : $file:ident),*
        ) => {
            $(pub mod $file;)*
            $(use crate::game::role::$file::$name;)*

            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, PartialOrd, Ord)]
            #[serde(rename_all = "camelCase")]
            pub enum Role {
                $($name),*
            }
            impl Role {
                pub fn values() -> VecSet<Role> {
                    return vec_set![$(Role::$name),*];
                }
                pub fn default_state(&self) -> RoleState {
                    match self {
                        $(Self::$name => RoleState::$name($file::$name::default())),*
                    }
                }
                pub fn new_state(&self, game: &Game) -> RoleState {
                    match self {
                        $(Self::$name => RoleState::$name($file::$name::new_state(game))),*
                    }
                }
                pub fn maximum_count(&self, settings: &Settings) -> Option<u8> {
                    if let Some(ModifierState::CustomRoleLimits(custom_role_limits)) = settings.modifiers.get_modifier_inner(ModifierID::CustomRoleLimits) {
                        custom_role_limits.limits.get(&self).copied()
                    } else {
                        match self {
                            $(Self::$name => $file::MAXIMUM_COUNT),*
                        }
                    }
                }
                pub fn defense(&self) -> DefensePower {
                    match self {
                        $(Self::$name => $file::DEFENSE),*
                    }
                }
                pub fn role_list_generation_criteria(&self) -> Vec<GenerationCriterion> {
                    match self {
                        $(Self::$name => $file::$name::role_list_generation_criteria()),*
                    }
                }
            }

            #[derive(Clone, Debug, Serialize)]
            #[serde(tag = "type", rename_all = "camelCase")]
            pub enum ClientRoleStateEnum {
                $($name(<$name as RoleStateTrait>::ClientAbilityState)),*
            }

            // This does not need to implement Deserialize or PartialEq!
            // Use Role for those things!
            #[derive(Clone, Debug)]
            pub enum RoleState {
                $($name($file::$name)),*
            }
            impl RoleState {
                pub fn role(&self) -> Role {
                    match self {
                        $(Self::$name(_) => Role::$name),*
                    }
                }
                
                pub fn on_player_roleblocked(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, invisible: bool){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_player_roleblocked(game, midnight_variables, actor_ref, player, invisible)),*
                    }
                }
                pub fn on_visit_wardblocked(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_visit_wardblocked(game, midnight_variables, actor_ref, visit)),*
                    }
                }
                pub fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_midnight(game, midnight_variables, actor_ref, priority)),*
                    }
                }
                pub fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
                    match self {
                        $(Self::$name(role_struct) => role_struct.controller_parameters_map(game, actor_ref)),*
                    }
                }
                pub fn on_controller_selection_changed(self, game: &mut Game, actor_ref: PlayerReference, id: ControllerID){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_controller_selection_changed(game, actor_ref, id)),*
                    }
                }
                pub fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: ControllerInput){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_validated_ability_input_received(game, actor_ref, input_player, ability_input)),*
                    }
                }
                pub fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.convert_selection_to_visits(game, actor_ref)),*
                    }
                }
                pub fn send_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap{
                    match self {
                        $(Self::$name(role_struct) => role_struct.send_player_chat_group_map(game, actor_ref)),*
                    }
                }
                pub fn receive_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap{
                    match self {
                        $(Self::$name(role_struct) => role_struct.receive_player_chat_group_map(game, actor_ref)),*
                    }
                }
                pub fn default_revealed_groups(self) -> VecSet<InsiderGroupID>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.default_revealed_groups()),*
                    }
                }
                pub fn default_win_condition(self) -> WinCondition{
                    match self {
                        $(Self::$name(role_struct) => role_struct.default_win_condition()),*
                    }
                }
                pub fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_phase_start(game, actor_ref, phase)),*
                    }
                }
                pub fn on_conceal_role(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, concealed_player: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_conceal_role(game, actor_ref, player, concealed_player)),*
                    }
                }
                pub fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_ability_creation(game, actor_ref, event, fold, priority)),*
                    }
                }
                pub fn on_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, old: RoleState, new: RoleState){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_role_switch(game, actor_ref, player, old, new)),*
                    }
                }
                pub fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, fold: &mut (), priority: OnAbilityDeletionPriority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_ability_deletion(game, actor_ref, event, fold, priority)),*
                    }
                }
                pub fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_any_death(game, actor_ref, dead_player_ref)),*
                    }
                }
                pub fn on_grave_added(self, game: &mut Game, actor_ref: PlayerReference, grave: GraveReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_grave_added(game, actor_ref, grave)),*
                    }
                }
                pub fn on_game_start(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_game_start(game, actor_ref)),*
                    }
                }
                pub fn on_game_ending(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_game_ending(game, actor_ref)),*
                    }
                }
                pub fn get_client_ability_state(self, game: &Game, actor_ref: PlayerReference) -> ClientRoleStateEnum {
                    match self {
                        $(Self::$name(role_struct) => ClientRoleStateEnum::$name(role_struct.get_client_ability_state(game, actor_ref))),*
                    }
                }
                pub fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_whisper(game, actor_ref, event, fold, priority)),*
                    }
                }
            }
            $(
                impl From<$file::$name> for RoleState where $name: RoleStateTrait {
                    fn from(role_struct: $file::$name) -> Self {
                        RoleState::$name(role_struct)
                    }
                }
            )*
        }
    }
    pub(super) use roles;
}
impl Role{
    pub fn possession_immune(&self)->bool{
        matches!(self, 
            | Role::Bouncer
            | Role::Veteran | Role::Medium
            | Role::Transporter | Role::Retributionist
            | Role::Witch | Role::Doomsayer | Role::Scarecrow | Role::Warper | Role::Porter
            | Role::Necromancer 
        )
    }
}