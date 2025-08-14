use std::vec;

use rand::rng;
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};

use super::event::on_midnight::MidnightVariables;
use super::phase::PhaseType;
use super::Game;
use super::player::PlayerReference;
use super::role::Role;
use super::role_list::RoleSet;




#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Grave {
    pub player: PlayerReference,
    pub died_phase: GravePhase,
    pub day_number: u8,

    pub information: GraveInformation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GraveInformation {
    Obscured,
    #[serde(rename_all = "camelCase")]
    Normal{
        role: Role,
        will: String,
        death_cause: GraveDeathCause,
        death_notes: Vec<String>,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "killers")]
pub enum GraveDeathCause {
    None,
    Execution,
    LeftTown,
    BrokenHeart,
    Killers(Vec<GraveKiller>)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "value")]
pub enum GraveKiller {
    RoleSet(RoleSet),
    Role(Role),
    Suicide,
    Quit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum GravePhase {
    Day,
    Night
}
impl GravePhase{
    pub fn from_phase_type(phase: PhaseType)->Self{
        match phase {
            PhaseType::Night => Self::Night,
            _ => Self::Day
        }
    }
}

impl Grave{

    pub fn role(&self)->Option<Role>{
        match self.information {
            GraveInformation::Obscured => None,
            GraveInformation::Normal { role, .. } => Some(role),
        }
    }


    pub fn from_player_night(game: &Game, midnight_variables: &MidnightVariables, player_ref: PlayerReference) -> Grave {
        let mut killers = player_ref.night_grave_killers(midnight_variables).clone();
        killers.shuffle(&mut rng());
        Grave {
            player: player_ref,
            died_phase: GravePhase::Night,
            day_number:  game.phase_machine.day_number,
            information: GraveInformation::Normal{
                role: player_ref.night_grave_role(midnight_variables).unwrap_or(player_ref.role(game)),
                will: player_ref.night_grave_will(midnight_variables).clone(),
                death_cause: GraveDeathCause::Killers(killers),
                death_notes: player_ref.night_grave_death_notes(midnight_variables).clone()
            },
        }
    }
    pub fn from_player_lynch(game: &Game, player_ref: PlayerReference) -> Grave {
        Grave { 
            player: player_ref,
            died_phase: GravePhase::Day, 
            day_number: game.phase_machine.day_number,
            information: GraveInformation::Normal{
                role: player_ref.role(game), 
                death_cause: GraveDeathCause::Execution, 
                will: player_ref.alibi(game).to_owned(), 
                death_notes: vec![]
            }
        }
    }

    pub fn from_player_suicide(game: &Game, player_ref: PlayerReference) -> Grave {
        Grave {
            player: player_ref,
            died_phase: GravePhase::from_phase_type(game.current_phase().phase()), 
            day_number: game.phase_machine.day_number,
            information: GraveInformation::Normal { 
                role: player_ref.role(game), 
                death_cause: GraveDeathCause::Killers(vec![GraveKiller::Suicide]), 
                death_notes: vec![],
                will: player_ref.alibi(game).to_owned(), 

            }
        }
    }

    pub fn from_player_leave_town(game: &Game, player_ref: PlayerReference) -> Grave {
        Grave {
            player: player_ref,
            died_phase: GravePhase::from_phase_type(game.current_phase().phase()), 
            day_number: game.phase_machine.day_number,
            information: GraveInformation::Normal { 
                role: player_ref.role(game), 
                death_cause: GraveDeathCause::LeftTown, 
                will: player_ref.alibi(game).to_owned(), 
                death_notes: vec![]
            }
        }
    }
    
    pub fn from_broken_heart(game: &Game, player_ref: PlayerReference) -> Grave {
        Grave {
            player: player_ref,
            died_phase: GravePhase::from_phase_type(game.current_phase().phase()), 
            day_number: game.phase_machine.day_number,
            information: GraveInformation::Normal { 
                role: player_ref.role(game),
                death_cause: GraveDeathCause::BrokenHeart, 
                will: player_ref.alibi(game).to_owned(),
                death_notes: vec![]
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct GraveReference{
    index: u8
}
impl GraveReference{
    pub fn new(game: &Game, index: u8)->Option<GraveReference> {
        if (index as usize) < game.graves.len() {
            Some(GraveReference { index })
        }else{
            None
        }
    }
    pub fn deref(self, game: &Game)->&Grave{
        unsafe {
            game.graves.get_unchecked(self.index as usize)
        }
    }
    pub fn deref_mut(self, game: &mut Game)->&mut Grave{
        unsafe {
            game.graves.get_unchecked_mut(self.index as usize)
        }
    }
}