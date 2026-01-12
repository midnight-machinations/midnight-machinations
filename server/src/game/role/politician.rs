use serde::Serialize;
use crate::game::prelude::*;


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
        
        EnfranchiseComponent::enfranchise(game, actor_ref, 3);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Politician, 0))
            .available_selection(AvailableUnitSelection)
            .add_grayed_out_condition(
                actor_ref.ability_deactivated_from_death(game) ||
                EnfranchiseComponent::enfranchised(game, actor_ref) || 
                PhaseType::Night == game.current_phase().phase() ||
                PhaseType::Briefing == game.current_phase().phase()
            )
            .dont_save()
            .allow_players([actor_ref])
            .build_map()
    }
    fn on_ability_deletion(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityDeletion, _fold: &mut (), priority: OnAbilityDeletionPriority) {
        if !event.id.is_players_role(actor_ref, Role::Politician) || priority != OnAbilityDeletionPriority::BeforeSideEffect {return;}
        EnfranchiseComponent::unenfranchise(game, actor_ref);
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        Self::check_for_lose_leave_town(&self, game, actor_ref);

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
                    actor_ref.edit_role_ability_helper(game, self);
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
        self.check_for_and_start_countdown(game, actor_ref);
    }

    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, _fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::SideEffect || !event.id.is_players_role(actor_ref, Role::Politician) {return}
        self.check_for_and_start_countdown(game, actor_ref);
    }

    fn default_win_condition(self) -> WinCondition where RoleState: From<Self> {
        WinCondition::GameConclusionReached{win_if_any: vec![GameConclusion::Politician].into_iter().collect()}
    }
    
    fn on_whisper(self, game: &mut Game, actor_ref: PlayerReference, event: &OnWhisper, fold: &mut WhisperFold, priority: WhisperPriority) {
        if priority == WhisperPriority::Cancel && (
            event.sender == actor_ref || 
            event.receiver == actor_ref
        ) && EnfranchiseComponent::enfranchised(game, actor_ref) {
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
    fn check_for_lose_leave_town(&self, game: &mut Game, actor_ref: PlayerReference){
        if
            !self.state.countdown_started() &&
            actor_ref.alive(game) &&
            PlayerReference::all_players(game)
                .filter(|p|p.keeps_game_running(game))
                .all(|p|!p.win_condition(game).is_loyalist_for(GameConclusion::Town))

        {
            actor_ref.die_and_add_grave(game, Grave::from_player_suicide(game, actor_ref));
        }
    }

    fn check_for_and_start_countdown(mut self, game: &mut Game, actor_ref: PlayerReference){
        if !actor_ref.alive(game) || self.state.countdown_started() {
            return; 
        }
        
        if Self::should_start_countdown(&self, game, actor_ref) {
            Politician::start_countdown(&mut self, game);
        }

        actor_ref.edit_role_ability_helper(game, self);
    }

    fn should_start_countdown(&self, game: &Game, actor_ref: PlayerReference)->bool{
        !self.state.countdown_started() &&
        actor_ref.alive(game) &&
        PlayerReference::all_players(game)
            .filter(|p|*p != actor_ref)
            .filter(|p|p.keeps_game_running(game))
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
            if !player.alive(game) || player.win_condition(game).is_loyalist_for(GameConclusion::Politician) {continue}
            
            let mut grave = Grave::from_player_lynch(game, player);
            if let GraveInformation::Normal{death_cause, ..} = &mut grave.information {
                *death_cause = GraveDeathCause::Killers(vec![GraveKiller::Role(Role::Politician)]);
            }
            player.die_and_add_grave(game, grave);
            
        }
    }
}