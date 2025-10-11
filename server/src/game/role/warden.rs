use serde::Serialize;
use crate::game::controllers::{AvailableBooleanSelection, AvailablePlayerListSelection, BooleanSelection};
use crate::game::components::insider_group::InsiderGroupID;
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::vec_map::VecMap;
use crate::{game::attack_power::AttackPower, vec_set::VecSet};
use crate::game::chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap};
use crate::game::components::graves::grave::GraveKiller;
use crate::game::phase::PhaseType;
use crate::game::attack_power::DefensePower;
use crate::game::player::PlayerReference;
use crate::game::abilities_component::ability_id::AbilityID;
use crate::game::Game;
use super::{ControllerID, ControllerParametersMap, PlayerListSelection, Role, RoleStateTrait};

#[derive(Clone, Debug, Serialize, Default)]
pub struct Warden{
    pub players_in_prison: VecSet<PlayerReference>,
    pub charges: u8,
}

const MAX_PLAYERS_IN_PRISON: u8 = 3;


pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::Armored;

impl RoleStateTrait for Warden {
    type ClientAbilityState = Warden;
    fn on_midnight(mut self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        if game.day_number() == 1 {return}

        match priority {
            OnMidnightPriority::Roleblock => {
                self.players_in_prison
                    .into_iter()
                    .filter(|p|*p != actor_ref)
                    .for_each(|p|p.roleblock(game, midnight_variables, true));
            }
            OnMidnightPriority::Kill => {
                if actor_ref.night_blocked(midnight_variables) {return}

                let players_to_kill = self.players_to_kill(game, actor_ref);

                Self::kill_players(
                    game,
                    midnight_variables,
                    actor_ref, 
                    if self.reached_charges_threshold(game) || self.players_in_prison.count() == 1 {
                        &self.players_in_prison
                    }else{
                        &players_to_kill
                    }
                );

                if self.all_prisoners_cooperated(&players_to_kill) {
                    self.increment_charges();

                    actor_ref.edit_role_ability_helper(game, self);
                }
            },
            _ => {}
        }
    }
    fn send_player_chat_group_map(self, game: &Game, _actor_ref: PlayerReference) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        
        if game.current_phase().phase() == PhaseType::Night {
            for target in self.players_in_prison {
                out.insert(target, ChatGroup::Warden);
            }
        }
        out
    }
    fn receive_player_chat_group_map(self, game: &Game, actor_ref: PlayerReference) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        out.insert(actor_ref, ChatGroup::Warden);
        if game.current_phase().phase() == PhaseType::Night {
            for target in self.players_in_prison {
                out.insert(target, ChatGroup::Warden);
            }
        }
        out
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::combine([
            // Put players in prison
            ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Warden, 0))
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|&p| p.alive(game))
                    .collect::<VecSet<_>>(),
                can_choose_duplicates: false,
                max_players: Some(MAX_PLAYERS_IN_PRISON)
            })
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game) || game.day_number() <= 1)
            .reset_on_phase_start(PhaseType::Night)
            .allow_players([actor_ref])
            .build_map(),


            ControllerParametersMap::combine(
                self.players_in_prison.iter().map(|&player|
                    ControllerParametersMap::builder(game)
                        .id(ControllerID::WardenCooperate{warden: actor_ref, player})
                        .available_selection(AvailableBooleanSelection)
                        .default_selection(BooleanSelection(true))
                        .allow_players([player])
                        .build_map()
                )
            )
        ])
    }
    fn on_phase_start(mut self, game: &mut Game, actor_ref: PlayerReference, phase: PhaseType){
        match phase {
            PhaseType::Night => {
                let Some(PlayerListSelection(players_in_prison)) = ControllerID::role(actor_ref, Role::Warden, 0)
                    .get_player_list_selection(game)
                    .cloned()
                    else {return};

                if actor_ref.ability_deactivated_from_death(game) || players_in_prison.iter().any(|p|!p.alive(game)) {return};
                
                self.players_in_prison = players_in_prison.into_iter().collect();
                
                actor_ref.edit_role_ability_helper(game, self.clone());

                game.add_message_to_chat_group(
                    crate::game::chat::ChatGroup::Warden,
                    ChatMessageVariant::WardenPlayersImprisoned{
                        players: self.players_in_prison.clone()
                    }
                );
                for &player in self.players_in_prison.iter(){
                    InsiderGroupID::send_message_in_available_insider_chat_or_private(
                        game,
                        player,
                        ChatMessageVariant::WardenPlayersImprisoned{
                            players: self.players_in_prison.clone()
                        },
                        false
                    );
                }
            },
            PhaseType::Obituary => {
                self.players_in_prison = VecSet::new();
                actor_ref.edit_role_ability_helper(game, self);
            },
            _ => {}
        }
    }
}

impl Warden {
    fn reached_charges_threshold(&self, game: &Game)->bool{
        self.charges >= game.num_players().div_ceil(2)
    }
    fn all_prisoners_cooperated(&self, players_to_kill: &VecSet<PlayerReference>) -> bool {
        players_to_kill.count() == 0 && self.players_in_prison.count() != 0
    }
    fn increment_charges(&mut self){
        self.charges = self.players_in_prison
            .count()
            .saturating_sub(1)
            .saturating_add(self.charges.into())
            .try_into()
            .unwrap_or(u8::MAX);
    }
    fn kill_players(game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, players: &VecSet<PlayerReference>){
        for player in players.iter() {
            player.try_night_kill_single_attacker(
                actor_ref,
                game, midnight_variables,
                GraveKiller::Role(Role::Warden),
                AttackPower::ArmorPiercing,
                true
            );
        }
    }
    fn players_to_kill(&self, game: &Game, actor_ref: PlayerReference)->VecSet<PlayerReference>{
        let players_cooperate_map: VecMap<PlayerReference, bool> = self.players_in_prison
            .iter()
            .map(|player|(
                *player,
                (ControllerID::WardenCooperate { warden: actor_ref, player: *player })
                    .get_boolean_selection(game)
                    .map(|b|b.0)
                    .unwrap_or(true)
            ))
            .collect();

        if players_cooperate_map.iter().all(|(_,cooperate)|*cooperate) {
            VecSet::new()
        }else if players_cooperate_map.iter().all(|(_,cooperate)|!cooperate) {
            self.players_in_prison.clone()
        }else{
            players_cooperate_map
                .into_iter()
                .filter_map(|(player,cooperate)|if cooperate {Some(player)}else{None})
                .collect()
        }
    }
}
