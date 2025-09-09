use kira_selection::{AvailableKiraSelection, KiraSelection};
use serde::{Serialize, Deserialize};

use crate::game::attack_power::AttackPower;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::{attack_power::DefensePower, chat::ChatMessageVariant};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::player::PlayerReference;

use crate::game::Game;
use crate::vec_map::VecMap;
use crate::game::controllers::*;
use super::{Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Kira;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum KiraGuess{
    None,
    #[default] NonTown,

    Jailor, Villager,
    Detective, Lookout, Tracker, Psychic, Philosopher, Gossip, Auditor, Snoop, Spy, TallyClerk,
    Doctor, Bodyguard, Cop, Bouncer, Engineer, Armorsmith, Steward,
    Vigilante, Veteran, Marksman, Deputy, Rabblerouser,
    Escort, Medium, Retributionist, Reporter, Mayor, Transporter, Porter, Polymath, Courtesan
}
impl KiraGuess{
    fn convert_to_guess(role: Role)->Option<KiraGuess>{
        match role {
            Role::Jailor => Some(Self::Jailor),
            Role::Villager => Some(Self::Villager),

            Role::Detective => Some(Self::Detective),
            Role::Lookout => Some(Self::Lookout),
            Role::Tracker => Some(Self::Tracker),
            Role::Philosopher => Some(Self::Philosopher),
            Role::Psychic => Some(Self::Psychic),
            Role::Auditor => Some(Self::Auditor),
            Role::Snoop => Some(Self::Snoop),
            Role::Gossip => Some(Self::Gossip),
            Role::Spy => Some(Self::Spy),
            Role::TallyClerk => Some(Self::TallyClerk),

            Role::Doctor => Some(Self::Doctor),
            Role::Bodyguard => Some(Self::Bodyguard),
            Role::Cop => Some(Self::Cop),
            Role::Bouncer => Some(Self::Bouncer),
            Role::Engineer => Some(Self::Engineer),
            Role::Armorsmith => Some(Self::Armorsmith),
            Role::Steward => Some(Self::Steward),

            Role::Vigilante => Some(Self::Vigilante),
            Role::Veteran => Some(Self::Veteran),
            Role::Marksman => Some(Self::Marksman),
            Role::Deputy => Some(Self::Deputy),
            Role::Rabblerouser => Some(Self::Rabblerouser),

            Role::Escort => Some(Self::Escort),
            Role::Courtesan => Some(Self::Courtesan),
            Role::Medium => Some(Self::Medium),
            Role::Retributionist => Some(Self::Retributionist),
            Role::Reporter => Some(Self::Reporter),
            Role::Mayor => Some(Self::Mayor),
            Role::Transporter => Some(Self::Transporter),
            Role::Porter => Some(Self::Porter),
            Role::Polymath => Some(Self::Polymath),

            //Mafia
            Role::Godfather | Role::Mafioso |
            Role::Counterfeiter | Role::Recruiter | Role::Impostor | Role::MafiaKillingWildcard |
            Role::Goon |
            Role::Hypnotist | Role::Blackmailer | Role::Cerenovous | Role::Informant | 
            Role::Necromancer | Role::Consort |
            Role::Mortician | Role::Framer | Role::Forger | 
            Role::Disguiser | Role::Reeducator |
            Role::Ambusher | Role::MafiaSupportWildcard => Some(Self::NonTown),

            //Neutral
            Role::Jester | Role::Revolutionary | Role::Politician |
            Role::Doomsayer | Role::Mercenary |
            Role::Witch | Role::Scarecrow | Role::Warper | Role::Kidnapper | Role::Chronokaiser |
            Role::Wildcard | Role::TrueWildcard | Role::Spiral |
            Role::SantaClaus | Role::Krampus => Some(Self::NonTown),
            Role::Martyr => None,

            //Fiends
            Role::Arsonist | Role::Werewolf | 
            Role::Ojo | Role::Puppeteer | Role::Pyrolisk | Role::Kira | 
            Role::SerialKiller | Role::Warden | Role::Yer |
            Role::FiendsWildcard => Some(Self::NonTown),
            
            //Cult
            Role::Apostle | Role::Disciple | Role::Zealot => Some(Self::NonTown),
        }
    }
    fn guess_matches_role(&self, role: Role)->bool{
        if let Some(guess) = Self::convert_to_guess(role) {
            *self == guess
        }else{
            false
        }
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
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if actor_ref.night_blocked(midnight_variables) {return;}
        if actor_ref.ability_deactivated_from_death(game) {return;}

        let Some(KiraSelection(selection)) = ControllerID::role(actor_ref, Role::Kira, 0).get_kira_selection(game)
            else {return};

        let result = KiraResult::new(selection.clone(), game);

        match priority {
            OnMidnightPriority::Kill if result.all_correct() => {
                if game.day_number() == 1 {return};
                
                for (player, (guess, result)) in result.guesses.iter(){
                    if player.alive(game) && *result == KiraGuessResult::Correct && *guess != KiraGuess::None {
                        player.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(super::Role::Kira), AttackPower::ArmorPiercing, true);
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