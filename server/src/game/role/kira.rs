use kira_selection::{AvailableKiraSelection, KiraSelection};
use serde::{Serialize, Deserialize};
use crate::game::components::attack::night_attack::NightAttack;
use crate::game::prelude::*;
use crate::vec_map::VecMap;

#[derive(Clone, Debug, Serialize, Default)]
pub struct Kira;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum KiraGuess{
    #[default] None,
    NonTown,
    #[serde(untagged)]
    Role(Role),
}
//[[1,"none"],[0,"villager"]]
impl KiraGuess{
    fn convert_to_guess(role: Role)->KiraGuess{
        if RoleSet::Town.get_roles().contains(&role) {
            KiraGuess::Role(role)
        } else {
            KiraGuess::NonTown
        }
    }
    fn guess_matches_role(&self, role: Role)->bool{
        *self == Self::convert_to_guess(role)
    }
    fn is_in_game(&self, game: &Game)->bool{
        PlayerReference::all_players(game).any(|player_ref| {
            let role = player_ref.role(game);
            self.guess_matches_role(role) && player_ref.alive(game)
        })
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct KiraResult {
    pub guesses: VecMap<PlayerReference, (KiraGuess, KiraGuessResult)>,
}
impl KiraResult{
    pub fn new(guesses: VecMap<PlayerReference, KiraGuess>, game: &Game)->Self{
        Self{
            guesses: guesses.into_iter().map(|(player_ref, guess)|{
                let result = if guess.guess_matches_role(player_ref.role(game)){
                    KiraGuessResult::Correct
                }else if guess.is_in_game(game) {
                    KiraGuessResult::WrongSpot
                }else{
                    KiraGuessResult::NotInGame
                };
                (player_ref, (guess, result))
            }).collect()
        }
    }
    pub fn all_correct(&self)->bool{
        self.guesses.iter().all(|(_, (guess, result))| 
            *result == KiraGuessResult::Correct || *guess == KiraGuess::None
        )
    }
}
impl Ord for KiraResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.guesses.len().cmp(&other.guesses.len())
    }
}
impl PartialOrd for KiraResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KiraGuessResult {
    Correct,    //green
    NotInGame,  //black
    WrongSpot,  //yellow
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct KiraControllerInput(Vec<(PlayerReference, KiraGuess)>);

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Kira {
    type ClientAbilityState = Kira;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if actor_ref.night_blocked(midnight_variables) {return;}
        if actor_ref.ability_deactivated_from_death(game) {return;}

        let Some(KiraSelection(selection)) = ControllerID::role(actor_ref, Role::Kira, 0).get_kira_selection(game)
            else {return};

        let result = KiraResult::new(selection.clone(), game);

        match priority {
            OnMidnightPriority::Kill => {
                if !result.all_correct() {return}
                if game.day_number() == 1 {return}
                
                for (player, (guess, result)) in result.guesses.iter(){
                    if player.alive(game) && *result == KiraGuessResult::Correct && *guess != KiraGuess::None {
                        NightAttack::new()
                            .attackers([actor_ref])
                            .grave_killer(Role::Kira)
                            .power(AttackPower::ArmorPiercing)
                            .leave_death_note()
                            .attack(game, midnight_variables, *player);
                    }
                }
            },
            OnMidnightPriority::Investigative => {
                actor_ref.push_night_message(midnight_variables, ChatMessageVariant::KiraResult { result });
            },
            _ => {},
        }    
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        match PlayerReference::all_players(game).filter(|p|p.alive(game)).count().saturating_sub(1).try_into() {
            Ok(count) => {
                ControllerParametersMap::builder(game)
                    .id(ControllerID::role(actor_ref, Role::Kira, 0))
                    .available_selection(AvailableKiraSelection::new(count))
                    .default_selection(KiraSelection::new(
                        PlayerReference::all_players(game)
                            .filter(|p|p.alive(game) && *p != actor_ref)
                            .map(|p|(p, KiraGuess::None))
                            .collect()
                    ))
                    .allow_players([actor_ref])
                    .build_map()
            }
            Err(_) => {
                ControllerParametersMap::default()
            }
        }        
    }
}