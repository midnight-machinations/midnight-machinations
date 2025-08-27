use std::{ops::Div, time::Duration};

use serde::{Serialize, Deserialize};

use crate::game::{components::graves::{grave::Grave, Graves}, modifiers::{hidden_nomination_votes::HiddenNominationVotes, hidden_verdict_votes::HiddenVerdictVotes, ModifierType, Modifiers}};

use super::{
    chat::{ChatGroup, ChatMessageVariant},
    event::{
        before_phase_end::BeforePhaseEnd,
        on_midnight::{OnMidnight, MidnightVariables}, on_phase_start::OnPhaseStart, Event
    },
    player::PlayerReference, settings::PhaseTimeSettings, Game
};


#[derive(Clone, Copy, PartialEq, Debug, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum PhaseType {
    Briefing,
    Obituary,
    Discussion,
    Nomination,
    Adjournment,
    Testimony,
    Judgement,
    FinalWords,
    Dusk,
    Night,
    Recess
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum PhaseState {
    Briefing,
    Obituary { 
        #[serde(skip)]
        last_night: MidnightVariables
    },
    Discussion,
    #[serde(rename_all = "camelCase")]
    Nomination { trials_left: u8, nomination_time_remaining: Option<Duration> },
    #[serde(rename_all = "camelCase")]
    Adjournment {trials_left: u8},
    #[serde(rename_all = "camelCase")]
    Testimony { trials_left: u8, player_on_trial: PlayerReference, nomination_time_remaining: Option<Duration> },
    #[serde(rename_all = "camelCase")]
    Judgement { trials_left: u8, player_on_trial: PlayerReference, nomination_time_remaining: Option<Duration> },
    #[serde(rename_all = "camelCase")]
    FinalWords { player_on_trial: PlayerReference },
    Dusk,
    Night,
    Recess
}
impl PartialOrd for PhaseState{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PhaseState{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.phase().cmp(&other.phase())
    }
}

pub struct PhaseStateMachine {
    pub time_remaining: Option<Duration>,
    pub current_state: PhaseState,
    pub day_number: u8, // Hopefully nobody is having more than 256 days anyway
}

impl PhaseStateMachine {
    pub fn new(times: PhaseTimeSettings) -> Self {
        let current_state = PhaseState::Briefing;

        Self {
            time_remaining: times.get_time_for(current_state.phase()),
            day_number: 1,
            current_state,
        }
    }

    pub fn get_time_remaining(&self) -> Option<Duration> {
        self.time_remaining
    }

    pub fn set_time_remaining(&mut self, time: Option<Duration>) {
        self.time_remaining = time;
    }

    pub fn next_phase(game: &mut Game, force_next_phase: Option<PhaseState>) {
        BeforePhaseEnd::new(game.current_phase().phase()).invoke(game);
        let mut new_phase = PhaseState::end(game);

        if let Some(forced_new_phase) = force_next_phase {
            new_phase = forced_new_phase;
        }

        game.phase_machine.current_state = new_phase;
        game.phase_machine.time_remaining = PhaseStateMachine::get_phase_time_length(game, game.current_phase().phase());

        PhaseState::start(game);
        OnPhaseStart::new(game.current_phase().clone()).invoke(game);
    }

    pub fn get_phase_time_length(game: &Game, phase: PhaseType) -> Option<Duration> {
        let mut time = game.settings.phase_times.get_time_for(phase);
        //if there are less than 3 players alive then the game is sped up by 2x
        if PlayerReference::all_players(game).filter(|p|p.alive(game)).count() <= 3{
            time = time.map(|o|o.div(2));
        }

        if
            phase == PhaseType::Nomination &&
            !Modifiers::is_enabled(game, ModifierType::UnscheduledNominations)
        {
            time = time.map(|o|o.div(3));
        }

        time
    }
}

impl PhaseState {
    pub const fn phase(&self) -> PhaseType {
        match self {
            PhaseState::Briefing => PhaseType::Briefing,
            PhaseState::Obituary {..} => PhaseType::Obituary,
            PhaseState::Discussion => PhaseType::Discussion,
            PhaseState::Nomination {..} => PhaseType::Nomination,
            PhaseState::Adjournment { .. } => PhaseType::Adjournment,
            PhaseState::Testimony {..} => PhaseType::Testimony,
            PhaseState::Judgement {..} => PhaseType::Judgement,
            PhaseState::FinalWords {..} => PhaseType::FinalWords,
            PhaseState::Dusk => PhaseType::Dusk,
            PhaseState::Night => PhaseType::Night,
            PhaseState::Recess => PhaseType::Recess
        }
    }

    pub fn start(game: &mut Game) {

        
        game.add_message_to_chat_group(ChatGroup::All, 
            ChatMessageVariant::PhaseChange { 
                phase: game.current_phase().clone(),
                //need this if statement because the day number should be increased for obituary phase
                day_number: if game.current_phase().phase() != PhaseType::Obituary {
                    game.phase_machine.day_number
                } else {
                    game.phase_machine.day_number.saturating_add(1)
                }
            }
        );
        
        match game.current_phase().clone() {
            PhaseState::Obituary { last_night } => {
                for player in PlayerReference::all_players(game){
                    let Some(role_state) = player.night_convert_role_to(&last_night).clone() else {continue};
                    player.set_role(game, role_state);
                }

                for player_ref in PlayerReference::all_players(game) {
                    if player_ref.night_died(&last_night) {
                        Graves::add_grave(game, Grave::from_player_night(game, &last_night, player_ref));
                    }
                }
                for player_ref in PlayerReference::all_players(game) {
                    if player_ref.night_died(&last_night) {
                        player_ref.die(game);
                    }
                }

                game.phase_machine.day_number = game.phase_machine.day_number.saturating_add(1);
            },
            PhaseState::Nomination { trials_left, nomination_time_remaining } => {
                game.phase_machine.set_time_remaining(nomination_time_remaining);

                let required_votes = game.nomination_votes_required();
                game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::TrialInformation { required_votes, trials_left });
                
                game.send_player_votes();
            },
            PhaseState::Testimony { player_on_trial, .. } => {

                if !HiddenNominationVotes::nomination_votes_are_hidden(game) {
                    game.add_message_to_chat_group(ChatGroup::All, 
                        ChatMessageVariant::PlayerNominated {
                            player_index: player_on_trial.index(),
                            players_voted: PlayerReference::all_players(game)
                            .filter(|player_ref| player_ref.chosen_vote(game) == Some(player_on_trial))
                            .map(|player_ref| player_ref.index())
                            .collect()
                        }
                    );
                }
            },
            PhaseState::Briefing 
            | PhaseState::Night
            | PhaseState::Discussion
            | PhaseState::Judgement { .. } 
            | PhaseState::FinalWords { .. }
            | PhaseState::Dusk
            | PhaseState::Recess 
            | PhaseState::Adjournment { .. }
            => {}
        }
        
    }
    
    /// Returns what phase should come next
    pub fn end(game: &mut Game) -> PhaseState {
        match *game.current_phase() {
            PhaseState::Briefing => {
                Self::Dusk
            },
            PhaseState::Obituary { .. }=> {
                Self::Discussion
            },
            PhaseState::Discussion => {
                Self::Nomination {
                    trials_left: 3,
                    nomination_time_remaining: PhaseStateMachine::get_phase_time_length(game, PhaseType::Nomination)
                }
            },
            PhaseState::Nomination {trials_left, ..} => {
                if Modifiers::is_enabled(game, ModifierType::UnscheduledNominations){
                    if trials_left == 0 {
                        return Self::Dusk
                    } else {
                        return Self::Adjournment { trials_left: 0 } //you only get an adjournment that is the length of the normal one rather than more than one adjournment or 1 longer one if you have more than 1 trial remaining
                    }
                }
                
                if let Some(player_on_trial) = game.count_nomination_and_start_trial(false){    
                    Self::Testimony{
                        trials_left: trials_left.saturating_sub(1), 
                        player_on_trial, 
                        nomination_time_remaining: PhaseStateMachine::get_phase_time_length(game, PhaseType::Nomination)
                    }
                } else if trials_left > 0 {
                    Self::Adjournment {trials_left: trials_left.saturating_sub(1)}
                } else {
                    Self::Adjournment {trials_left}
                }
            },
            PhaseState::Adjournment {trials_left} => {
                if trials_left == 0 {
                    Self::Dusk
                } else {
                    Self::Nomination {
                        trials_left,
                        nomination_time_remaining: PhaseStateMachine::get_phase_time_length(game, PhaseType::Nomination)
                    }
                }
            }
            PhaseState::Testimony { trials_left, player_on_trial, nomination_time_remaining } => {
                Self::Judgement { trials_left, player_on_trial, nomination_time_remaining }
            },
            PhaseState::Judgement { trials_left, player_on_trial, nomination_time_remaining } => {


                let (guilty, innocent) = game.count_verdict_votes(player_on_trial);

                if !HiddenVerdictVotes::verdict_votes_are_hidden(game) {
                    
                    game.add_messages_to_chat_group(ChatGroup::All, 
                        PlayerReference::all_players(game)
                            .filter(|player_ref|{
                                player_ref.alive(game) && *player_ref != player_on_trial
                            })
                            .map(|player_ref|
                                ChatMessageVariant::JudgementVerdict{
                                    voter_player_index: player_ref.index(),
                                    verdict: player_ref.verdict(game)
                                }
                            )
                            .collect()
                    );


                    game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::TrialVerdict{ 
                        player_on_trial: player_on_trial.index(), 
                        innocent, guilty 
                    });
                }
                

                let hang = if Modifiers::is_enabled(game, ModifierType::TwoThirdsMajority) {
                    innocent <= guilty.div(2)
                } else {
                    innocent < guilty
                };
                
                if hang {
                    Self::FinalWords { player_on_trial }
                } else if trials_left == 0 {
                    Self::Dusk
                }else{
                    Self::Nomination { trials_left, nomination_time_remaining }
                }
            },
            PhaseState::FinalWords { player_on_trial } => {
                player_on_trial.die_and_add_grave(game, Grave::from_player_lynch(game, player_on_trial));

                Self::Dusk
            },
            PhaseState::Dusk => {
                Self::Night
            },
            PhaseState::Night => {
                Self::Obituary { last_night: OnMidnight::new().invoke(game) }
            },
            PhaseState::Recess => Self::Recess
        }
    }
    
    pub fn is_day(&self) -> bool {
        self.phase() != PhaseType::Night
    }

    pub fn is_night(&self) -> bool {
        self.phase() == PhaseType::Night
    }
}