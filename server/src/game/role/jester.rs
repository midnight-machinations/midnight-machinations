use rand::seq::IndexedRandom;
use serde::Serialize;
use crate::game::{components::attack::night_attack::NightAttack, prelude::*, verdict::Verdict};

#[derive(Clone, Debug, Default)]
pub struct Jester {
    executed_yesterday: bool,
    won: bool,

    num_deaths_before_lose: u8,
}

#[derive(Clone, Serialize, Debug)]
pub struct ClientRoleState;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Jester {
    type ClientAbilityState = ClientRoleState;
    fn new_state(game: &mut Game) -> Self {
        Self {
            executed_yesterday: false,
            won: false,
            num_deaths_before_lose: PlayerReference::all_players(game)
                .filter(|p|!p.win_condition(game).friends_with_conclusion(GameConclusion::Town))
                .count()
                .div_ceil(3)
                .try_into()
                .unwrap_or(u8::MAX),
            }
    }
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::TopPriority {return;}
        if actor_ref.alive(game) {return;}
        if !self.executed_yesterday {return}

        let target_ref = if let Some(target_ref) = ControllerID::role(actor_ref, Role::Jester, 0)
            .get_player_list_selection(game)
            .and_then(|s|s.0.first())
        {
            *target_ref
        }else{
            let all_killable_players: Vec<PlayerReference> = PlayerReference::all_players(game)
                .filter(|player_ref|{
                    player_ref.alive(game) &&
                    *player_ref != actor_ref &&
                    player_ref.verdict(game) != Verdict::Innocent
                })
                .collect();

            let Some(target_ref) = all_killable_players
                .choose(&mut game.rng) else {return};
            
            *target_ref
        };
        
        NightAttack::new()
            .attackers([actor_ref])
            .grave_killer(Role::Jester)
            .power(AttackPower::ProtectionPiercing)
            .leave_death_note()
            .attack(game, midnight_variables, target_ref);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Jester, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|p| *p != actor_ref)
                    .filter(|player| 
                        player.alive(game) &&
                        player.verdict(game) != Verdict::Innocent
                    )
                    .collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .add_grayed_out_condition(!self.executed_yesterday)
            .reset_on_phase_start(PhaseType::Obituary)
            .allow_players([actor_ref])
            .build_map()
    }
    fn on_phase_start(self, game: &mut Game, actor_ref: PlayerReference, _phase: PhaseType){
        match game.current_phase() {
            PhaseState::FinalWords { player_on_trial } if *player_on_trial == actor_ref => {
                actor_ref.edit_role_ability_helper(game, Jester { 
                    executed_yesterday: true,
                    won: true,
                    ..self
                });
            }
            PhaseState::Obituary { .. } => {
                actor_ref.edit_role_ability_helper(game, Jester { 
                    executed_yesterday: false,
                    ..self
                });
            }
            _ => {}
        }
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, dead_player_ref: PlayerReference) {
        if dead_player_ref.win_condition(game).friends_with_conclusion(GameConclusion::Town) {return}

        let new_num_deaths_before_lose = self.num_deaths_before_lose.saturating_sub(1);
        actor_ref.edit_role_ability_helper(game, Jester { 
            num_deaths_before_lose: new_num_deaths_before_lose,
            ..self
        });
        
        if new_num_deaths_before_lose == 0 {
            actor_ref.die_and_add_grave(game, Grave::from_player_suicide(game, actor_ref));
        }
    }
}
impl GetClientAbilityState<ClientRoleState> for Jester {
    fn get_client_ability_state(self, _game: &Game, _actor_ref: PlayerReference) -> ClientRoleState {
        ClientRoleState
    }
}

impl Jester {
    pub fn won(&self) -> bool {
        self.won
    }
}