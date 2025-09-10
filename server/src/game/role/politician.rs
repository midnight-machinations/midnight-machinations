use serde::Serialize;
use crate::game::controllers::AvailableUnitSelection;
use crate::game::attack_power::DefensePower;
use crate::game::chat::{ChatGroup, ChatMessageVariant};
use crate::game::components::graves::grave::Grave;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_ability_deletion::{OnAbilityDeletion, OnAbilityDeletionPriority};
use crate::game::event::on_whisper::{OnWhisper, WhisperFold, WhisperPriority};
use crate::game::components::enfranchise::Enfranchise;
use crate::game::game_conclusion::GameConclusion;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;
use crate::game::components::win_condition::WinCondition;
use crate::game::Game;

use super::{ControllerID, ControllerParametersMap, GetClientAbilityState, Role, RoleState, RoleStateTrait};


#[derive(Debug, Clone, Default)]
pub struct Politician{
    state: PoliticianState,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PoliticianState{
    #[default]
    None,
    CountdownStarted,
    FinalNomination,
    Finished
}
impl PoliticianState {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn countdown_started(&self) -> bool {
        match self {
            PoliticianState::None => false,
            PoliticianState::CountdownStarted => true,
            PoliticianState::FinalNomination => true,
            PoliticianState::Finished => false,
        }
    }
    pub fn final_nomination(&self) -> bool {
        match self {
            PoliticianState::None => false,
            PoliticianState::CountdownStarted => false,
            PoliticianState::FinalNomination => true,
            PoliticianState::Finished => false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Politician {
    type ClientAbilityState = ClientRoleState;
    fn on_validated_ability_input_received(self, game: &mut Game, actor_ref: PlayerReference, input_player: PlayerReference, ability_input: super::ControllerInput) {
        if actor_ref != input_player {return;}
        if ability_input.id() != ControllerID::role(actor_ref, Role::Politician, 0) {
            return;
        }
        
        Enfranchise::enfranchise(game, actor_ref);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Politician, 0))
            .available_selection(AvailableUnitSelection)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                Enfranchise::enfranchised(game, actor_ref) || 
                PhaseType::Night == game.current_phase().phase() ||
                PhaseType::Briefing == game.current_phase().phase()
            )
            .dont_save()
            .allow_players([actor_ref])
            .build_map()
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Politician) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        Enfranchise::unenfranchise(game, actor_ref);
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        Self::check_and_leave_town(&self, game, actor_ref);

        if self.state.countdown_started() && actor_ref.alive(game) {
            //for skipping phases
            // this litterally causes the entire server to crash
            // match phase {
            //     PhaseType::Briefing | PhaseType::Nomination | PhaseType::Testimony | 
            //     PhaseType::Judgement | PhaseType::FinalWords | PhaseType::Recess => {}

            //     PhaseType::Obituary | PhaseType::Discussion | PhaseType::Dusk | PhaseType::Night => {
            //         game.phase_machine.time_remaining = Duration::from_secs(0);
            //     }
            // }

            match phase {
                PhaseType::Nomination => {
                    self.state = PoliticianState::FinalNomination;
                    actor_ref.set_role_state(game, self);
                },
                PhaseType::Dusk => {
                    if self.state == PoliticianState::FinalNomination {
                        Politician::kill_all(game);
                    }
                },
                _ => {}
            }

        }
    }
    
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
        self.check_and_start_countdown(game, actor_ref);
    }

    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Politician) || fold.cancelled {return}
        self.check_and_start_countdown(game, actor_ref);
    }

    fn default_win_condition(self) -> WinCondition where RoleState: From<Self> {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Politician].into_iter().collect()}
    }
    
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if priority == WhisperPriority::Cancel && (
            event.sender == actor_ref || 
            event.receiver == actor_ref
        ) && Enfranchise::enfranchised(game, actor_ref) {
            fold.cancelled = true;
            fold.hide_broadcast = true;
        }
    }
}

impl GetClientAbilityState<ClientRoleState> for Politician {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Politician {
    fn check_and_leave_town(&self, game: &mut Game, actor_ref: PlayerReference){
        if
            !self.state.countdown_started() &&
            actor_ref.alive(game) &&
            PlayerReference::all_players(game)
                .filter(|p|p.alive(game))
                .filter(|p|p.keeps_game_running(game))
                .all(|p|
                    !p.win_condition(game).is_loyalist_for(GameConclusion::Town)
                )

        {
            actor_ref.die_and_add_grave(game, Grave::from_player_leave_town(game, actor_ref));
        }
    }

    fn check_and_start_countdown(mut self, game: &mut Game, actor_ref: PlayerReference){
        if !actor_ref.alive(game) || self.state.countdown_started() {
            return; 
        }
        
        if Self::should_start_countdown(&self, game, actor_ref) {
            Politician::start_countdown(&mut self, game);
        }

        actor_ref.set_role_state(game, self);
    }


    fn should_start_countdown(&self, game: &Game, actor_ref: PlayerReference)->bool{
        !self.state.countdown_started() &&
        actor_ref.alive(game) &&
            PlayerReference::all_players(game)
            .filter(|p|*p != actor_ref)
            .filter(|p|p.keeps_game_running(game))
            .filter(|p|p.alive(game))
            .all(|player| {
                player.win_condition(game).is_loyalist_for(GameConclusion::Town)
            })
    }

    fn start_countdown(&mut self, game: &mut Game){
        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::PoliticianCountdownStarted);
        
        // causes the entire server to crash
        // if game.current_phase().phase() != PhaseType::Nomination {
        //     game.phase_machine.time_remaining = Duration::from_secs(0);
        // }
        self.state = PoliticianState::CountdownStarted;
    }

    fn kill_all(game: &mut Game){
        for player in PlayerReference::all_players(game){
            if player.alive(game) && !player.win_condition(game).is_loyalist_for(GameConclusion::Politician) {
                player.die_and_add_grave(game, Grave::from_player_leave_town(game, player));
            }
        }
    }
}