#![allow(clippy::single_match, reason = "May add more cases for more priorities later")]

use std::collections::HashSet;
use crate::vec_set::{vec_set, VecSet};

use crate::game::player::PlayerReference;
use crate::game::visit::Visit;
use crate::game::Game;
use crate::game::chat::ChatGroup;
use crate::game::phase::PhaseType;
use crate::game::attack_power::DefensePower;

use serde::{Serialize, Deserialize};

use super::components::win_condition::WinCondition;
use super::{
    ability_input::*, components::{insider_group::InsiderGroupID, night_visits::NightVisits},
    event::{on_midnight::{MidnightVariables, OnMidnightPriority}, on_whisper::{OnWhisper, WhisperFold, WhisperPriority}},
    grave::GraveReference, visit::VisitTag,
};

pub trait GetClientRoleState<CRS> {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> CRS;
}
//Automatically implement this for the case where RoleState = ClientRoleState
impl<T> GetClientRoleState<T> for T {
    fn get_client_role_state(self, _game: &Game, _actor_ref: PlayerReference) -> T {
        self
    }
}

pub trait RoleStateImpl: Clone + std::fmt::Debug + Default + GetClientRoleState<<Self as RoleStateImpl>::ClientRoleState> {
    type ClientRoleState: Clone + std::fmt::Debug + Serialize;
    fn on_midnight(self, _game: &mut Game, _midnight_variables: &mut MidnightVariables, _actor_ref: PlayerReference, _priority: OnMidnightPriority) {}

    fn controller_parameters_map(self, _game: &Game, _actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::default()
    }
    fn on_controller_selection_changed(self, _game: &mut Game, _actor_ref: PlayerReference, _id: ControllerID) {}
    fn on_validated_ability_input_received(self, _game: &mut Game, _actor_ref: PlayerReference, _input_player: PlayerReference, _ability_input: AbilityInput) {}
    fn on_ability_input_received(self, _game: &mut Game, _actor_ref: PlayerReference, _input_player: PlayerReference, _ability_input: AbilityInput) {}

    fn convert_selection_to_visits(self, _game: &Game, _actor_ref: PlayerReference) -> Vec<Visit> {
        vec![]
    }

    fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_send_chat_groups(game, actor_ref, vec![])
    }
    fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup> {
        crate::game::role::common_role::get_current_receive_chat_groups(game, actor_ref)
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
    fn on_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn before_role_switch(self, _game: &mut Game, _actor_ref: PlayerReference, _player: PlayerReference, _new: RoleState, _old: RoleState) {}
    fn on_any_death(self, _game: &mut Game, _actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {}
    fn on_grave_added(self, _game: &mut Game, _actor_ref: PlayerReference, _grave: GraveReference) {}
    fn on_game_ending(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn on_game_start(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn before_initial_role_creation(self, _game: &mut Game, _actor_ref: PlayerReference) {}
    fn on_conceal_role(self, _game: &mut Game, _actor_ref: PlayerReference, _player: PlayerReference, _concealed_player: PlayerReference) {}
    fn on_player_roleblocked(self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, player: PlayerReference, _invisible: bool) {
        if player != actor_ref {return}

        NightVisits::retain(midnight_variables, |v|
            !matches!(v.tag, VisitTag::Role{..}) || v.visitor != actor_ref
        );
    }
    fn on_visit_wardblocked(self, _game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, visit: Visit) {
        if actor_ref != visit.visitor {return};

        NightVisits::retain(midnight_variables, |v|
            !matches!(v.tag, VisitTag::Role{..}) || v.visitor != actor_ref
        );
    }
    fn on_whisper(self, _game: &mut Game, _actor_ref: PlayerReference, _event: &OnWhisper, _fold: &mut WhisperFold, _priority: WhisperPriority) {}
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
    Coxswain : coxswain,
    Polymath : polymath,

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
    Informant: informant,
    MafiaWitch : mafia_witch,
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
    Wildcard : wild_card,
    TrueWildcard : true_wildcard,
    Martyr : martyr,
    SantaClaus : santa_claus,
    Krampus : krampus,

    Witch : witch,
    Scarecrow : scarecrow,
    Warper : warper,
    Kidnapper : kidnapper,
    Chronokaiser : chronokaiser,

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
                pub fn maximum_count(&self) -> Option<u8> {
                    match self {
                        $(Self::$name => $file::MAXIMUM_COUNT),*
                    }
                }
                pub fn defense(&self) -> DefensePower {
                    match self {
                        $(Self::$name => $file::DEFENSE),*
                    }
                }
            }

            #[derive(Clone, Debug, Serialize)]
            #[serde(tag = "type", rename_all = "camelCase")]
            pub enum ClientRoleStateEnum {
                $($name(<$name as RoleStateImpl>::ClientRoleState)),*
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
                pub fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: AbilityInput){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_validated_ability_input_received(game, actor_ref, input_player, ability_input)),*
                    }
                }
                pub fn on_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: AbilityInput){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_ability_input_received(game, actor_ref, input_player, ability_input)),*
                    }
                }
                pub fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.convert_selection_to_visits(game, actor_ref)),*
                    }
                }
                pub fn get_current_send_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.get_current_send_chat_groups(game, actor_ref)),*
                    }
                }
                pub fn get_current_receive_chat_groups(self, game: &Game, actor_ref: PlayerReference) -> HashSet<ChatGroup>{
                    match self {
                        $(Self::$name(role_struct) => role_struct.get_current_receive_chat_groups(game, actor_ref)),*
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
                pub fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_role_creation(game, actor_ref)),*
                    }
                }
                pub fn before_role_switch(self, game: &mut Game, actor_ref: PlayerReference, player: PlayerReference, old: RoleState, new: RoleState){
                    match self {
                        $(Self::$name(role_struct) => role_struct.before_role_switch(game, actor_ref, player, old, new)),*
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
                pub fn before_initial_role_creation(self, game: &mut Game, actor_ref: PlayerReference){
                    match self {
                        $(Self::$name(role_struct) => role_struct.before_initial_role_creation(game, actor_ref)),*
                    }
                }
                pub fn get_client_role_state(self, game: &Game, actor_ref: PlayerReference) -> ClientRoleStateEnum {
                    match self {
                        $(Self::$name(role_struct) => ClientRoleStateEnum::$name(role_struct.get_client_role_state(game, actor_ref))),*
                    }
                }
                pub fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority){
                    match self {
                        $(Self::$name(role_struct) => role_struct.on_whisper(game, actor_ref, event, fold, priority)),*
                    }
                }
            }
            $(
                impl From<$file::$name> for RoleState where $name: RoleStateImpl {
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
            | Role::Veteran | Role::Coxswain
            | Role::Transporter | Role::Retributionist
            | Role::Witch | Role::Doomsayer | Role::Scarecrow | Role::Warper | Role::Porter
            | Role::MafiaWitch | Role::Necromancer 
        )
    }
    pub fn has_innocent_aura(&self, game: &Game)->bool{
        match self {
            Role::Godfather => true,
            Role::Pyrolisk => {
                game.day_number() == 1
            },
            _ => false,
        }
    }
    pub fn has_suspicious_aura(&self, _game: &Game)->bool{
        false
    }
}