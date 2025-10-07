use serde::{Serialize, Deserialize};
use crate::game::abilities_component::ability_id::AbilityID;

use crate::game::attack_power::{AttackPower, DefensePower};
use crate::game::chat::ChatMessageVariant;
use crate::game::event::on_ability_creation::{OnAbilityCreation, OnAbilityCreationFold, OnAbilityCreationPriority};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::player::PlayerReference;

use crate::game::Game;

use super::jester::Jester;
use super::{GetClientAbilityState, Role, RoleState, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Doomsayer {
    pub guesses: [(PlayerReference, DoomsayerGuess); 3],
    pub won: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClientRoleState {
    guesses: [(PlayerReference, DoomsayerGuess); 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum DoomsayerGuess{
    #[default] NonTown,

    Jailor, Villager,
    // No TI
    Doctor, Bodyguard, Cop, Bouncer, Engineer, Armorsmith, Steward,
    Vigilante, Veteran, Marksman, Deputy, Rabblerouser,
    Escort, Medium, Retributionist, Reporter, Mayor, Porter, Transporter, Polymath, Courtesan, Nepotist
}
impl DoomsayerGuess{
    fn convert_to_guess(role: Role)->Option<DoomsayerGuess>{
        match role {
            Role::Jailor => Some(DoomsayerGuess::Jailor),
            Role::Villager => Some(DoomsayerGuess::Villager),

            Role::Detective | Role::Lookout | Role::Spy | 
            Role::Tracker | Role::Philosopher | Role::Psychic | 
            Role::Auditor | Role::Snoop | Role::Gossip | Role::TallyClerk => None, 

            Role::Doctor => Some(DoomsayerGuess::Doctor),
            Role::Bodyguard => Some(DoomsayerGuess::Bodyguard),
            Role::Cop => Some(DoomsayerGuess::Cop),
            Role::Bouncer => Some(DoomsayerGuess::Bouncer),
            Role::Engineer => Some(DoomsayerGuess::Engineer),
            Role::Armorsmith => Some(DoomsayerGuess::Armorsmith),
            Role::Steward => Some(DoomsayerGuess::Steward),

            Role::Vigilante => Some(DoomsayerGuess::Vigilante),
            Role::Veteran => Some(DoomsayerGuess::Veteran),
            Role::Marksman => Some(DoomsayerGuess::Marksman),
            Role::Deputy => Some(DoomsayerGuess::Deputy),
            Role::Rabblerouser => Some(DoomsayerGuess::Rabblerouser),

            Role::Escort => Some(DoomsayerGuess::Escort),
            Role::Courtesan => Some(DoomsayerGuess::Courtesan),
            Role::Medium => Some(DoomsayerGuess::Medium),
            Role::Retributionist => Some(DoomsayerGuess::Retributionist),
            Role::Reporter => Some(DoomsayerGuess::Reporter),
            Role::Mayor => Some(DoomsayerGuess::Mayor),
            Role::Nepotist => Some(DoomsayerGuess::Nepotist),
            Role::Porter => Some(DoomsayerGuess::Porter),
            Role::Transporter => Some(DoomsayerGuess::Transporter),
            Role::Polymath => Some(DoomsayerGuess::Polymath),

            //Mafia
            Role::Godfather | Role::Mafioso | 
            Role::Counterfeiter | Role::Recruiter | Role::Impostor | Role::MafiaKillingWildcard |
            Role::Goon |
            Role::Hypnotist | Role::Blackmailer | Role::Cerenovous | Role::Informant | 
            Role::Necromancer | Role::Consort |
            Role::Mortician | Role::Framer | Role::Forger | 
            Role::Disguiser | Role::Reeducator |
            Role::Ambusher | Role::MafiaSupportWildcard => Some(DoomsayerGuess::NonTown),

            //Neutral
            Role::Jester | Role::Revolutionary | Role::Politician |
            Role::Doomsayer | Role::Mercenary |
            Role::Chronokaiser | Role::Wildcard | Role::TrueWildcard | Role::SantaClaus | Role::Krampus |
            //Minion
            Role::Witch | Role::Scarecrow | Role::Warper | Role::Kidnapper | Role::Pawn | Role::Tailor |
            //other
            Role::Drunk => Some(DoomsayerGuess::NonTown),
            Role::Martyr => None,
           

            //Fiends
            Role::Arsonist | Role::Werewolf | 
            Role::Ojo | Role::Puppeteer | Role::Pyrolisk | Role::Kira |
            Role::SerialKiller | Role::Warden | Role::Yer | Role::Spiral |
            Role::FiendsWildcard => Some(DoomsayerGuess::NonTown),
            
            //Cult
            Role::Apostle | Role::Disciple | Role::Zealot => Some(DoomsayerGuess::NonTown),
        }
    }
    fn guess_matches_role(&self, role: Role)->bool{
        if let Some(guess) = Self::convert_to_guess(role) {
            *self == guess
        }else{
            false
        }
    }
}


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Doomsayer {
    type ClientAbilityState = ClientRoleState;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::TopPriority {return;}

        if actor_ref.night_blocked(midnight_variables) {return;}
        if !actor_ref.alive(game) {return;}


        let mut won = true;
        for (player, guess) in self.guesses.iter(){
            if 
                *player == actor_ref || //cant guess yourself
                !player.alive(game) || //cant guess dead player
                !guess.guess_matches_role(player.role(game)) || //cant guess wrong
                self.guesses.iter().filter(|(other_p, _other_g)|{
                    *other_p == *player
                }).count() != 1 //cant guess a player more than once
            {
                won = false;
                break;
            }
        };

        if won{
            actor_ref.add_private_chat_message(game, ChatMessageVariant::DoomsayerWon);
            self.guesses[0].0.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(super::Role::Doomsayer), AttackPower::ProtectionPiercing, true);
            self.guesses[1].0.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(super::Role::Doomsayer), AttackPower::ProtectionPiercing, true);
            self.guesses[2].0.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Role(super::Role::Doomsayer), AttackPower::ProtectionPiercing, true);
            actor_ref.try_night_kill_single_attacker(actor_ref, game, midnight_variables, GraveKiller::Suicide, AttackPower::ProtectionPiercing, false);
            actor_ref.set_role_state(game, RoleState::Doomsayer(Doomsayer { guesses: self.guesses, won: true }));
        }else{
            actor_ref.add_private_chat_message(game, ChatMessageVariant::DoomsayerFailed);
        }
    
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType) {
        Doomsayer::check_and_convert_to_jester(game, self, actor_ref);
    }
    fn on_ability_creation(self, game: &mut Game, actor_ref: PlayerReference, event: &OnAbilityCreation, fold: &mut OnAbilityCreationFold, priority: OnAbilityCreationPriority) {
        if priority != OnAbilityCreationPriority::CancelOrEdit{return}
        if event.id.is_players_role(actor_ref, Role::Doomsayer)
        {
            fold.cancelled = Doomsayer::check_and_convert_to_jester(game, self, actor_ref);
        }
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, _dead_player_ref: PlayerReference){
        Doomsayer::check_and_convert_to_jester(game, self, actor_ref);
    }
}
impl GetClientAbilityState<ClientRoleState> for Doomsayer {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState{
            guesses: self.guesses
        }
    }
}
impl Doomsayer{
    pub fn check_and_convert_to_jester(game: &mut Game, doomsayer: Doomsayer, actor_ref: PlayerReference)->bool{
        if
            !doomsayer.won && actor_ref.alive(game) &&
            PlayerReference::all_players(game).filter(|player|
                player.alive(game) && DoomsayerGuess::convert_to_guess(player.role(game)).is_some() && *player != actor_ref
            ).count() < 3
        {
            actor_ref.set_role_and_win_condition_and_revealed_group(game, RoleState::Jester(Jester::default()));
            true
        }else{
            false
        }
    }
    pub fn won(&self) -> bool {
        self.won
    }
}