use rand::seq::SliceRandom;
use crate::{
    game::{
        attack_power::{AttackPower, DefensePower}, chat::{ChatMessage, ChatMessageVariant}, components::{
            attack::night_attack::NightAttack, fragile_vest::FragileVests, graves::{grave::{Grave, GraveKiller}, Graves},
            insider_group::InsiderGroupID, night_visits::{NightVisitsIterator, Visits}, player_component::PlayerComponent,
            role::RoleComponent,
        },
        controllers::{ControllerID, PlayerListSelection},
        event::{
            on_any_death::OnAnyDeath, on_midnight::{MidnightVariables, OnMidnightPriority}, Event
        },
        role::{medium::Medium, RoleState},
        visit::Visit,
        Game
    },
    packet::ToClientPacket, vec_set::VecSet
};

use super::PlayerReference;

impl PlayerReference{
    #[expect(clippy::too_many_arguments, reason="this function is goated tho")]
    pub fn rampage(
        &self, game: &mut Game,
        midnight_variables: &mut MidnightVariables,
        attacker: PlayerReference,
        grave_killer: GraveKiller,
        attack: AttackPower,
        should_leave_death_note: bool,
        filter_visit: impl FnMut(&Visit) -> bool
    ){
        Visits::into_iter(midnight_variables)
            .filter(filter_visit)
            .with_target(*self)
            .with_direct()
            .map_visitor()
            .for_each(|p|{
                p.try_night_kill_single_attacker(
                    attacker,
                    game,
                    midnight_variables,
                    grave_killer.clone(),
                    attack,
                    should_leave_death_note
                );
            });
    }


    /// Returns true if attack overpowered defense
    pub fn try_night_kill_single_attacker(&self, attacker_ref: PlayerReference, game: &mut Game, midnight_variables: &mut MidnightVariables, grave_killer: GraveKiller, attack: AttackPower, should_leave_death_note: bool) -> bool {
        self.try_night_kill(
            vec![attacker_ref].into_iter().collect(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            should_leave_death_note
        )
    }
    pub fn try_night_kill(&self, attackers: VecSet<PlayerReference>, game: &mut Game, midnight_variables: &mut MidnightVariables, grave_killer: GraveKiller, attack_power: AttackPower, leave_death_note: bool) -> bool {
        NightAttack::new_attack(game, midnight_variables, true, *self, attackers, attack_power, leave_death_note, grave_killer)
    }
    pub fn try_night_kill_no_attacker(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, grave_killer: GraveKiller, attack: AttackPower) -> bool {
        self.try_night_kill(
            VecSet::new(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            false
        )
    }

    pub fn die_and_add_grave(&self, game: &mut Game, grave: Grave){
        if !self.alive(game) { return }
        Graves::add_grave(game, grave);
        self.die(game);
    }
    /// if the player is already dead, this does nothing
    pub fn die(&self, game: &mut Game){
        if !self.alive(game) { return }
        self.set_alive(game, false);
        self.add_private_chat_message(game, ChatMessageVariant::YouDied);
        OnAnyDeath::new(*self).invoke(game)
    }
    pub fn initial_role_creation(&self, game: &mut Game){
        self.set_win_condition(game, self.win_condition(game).clone());
        InsiderGroupID::set_player_insider_groups(
            InsiderGroupID::all_groups_with_player(game, *self), 
            game, *self
        );
        RoleComponent::set_role_without_ability(*self, game, self.role(game));
    }
    /// Swaps this persons role, sends them the role chat message, and makes associated changes
    pub fn set_role_win_con_insider_group(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        
        self.set_new_role(game, new_role_data, true);
    
        self.set_win_condition(game, self.role_state(game).clone().default_win_condition());
        
        InsiderGroupID::set_player_insider_groups(
            self.role_state(game).clone().default_revealed_groups(), 
            game, *self
        );
    }
    /// Swaps this persons role, sends them the role chat message, and makes associated changes
    pub fn set_role_win_con_insider_group_midnight(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        
        self.set_night_convert_role_to(midnight_variables, Some(new_role_data.clone()));
    
        self.set_win_condition(game, new_role_data.clone().default_win_condition());
        
        InsiderGroupID::set_player_insider_groups(
            new_role_data.clone().default_revealed_groups(), 
            game, *self
        );
    }
    
    
    pub fn normal_defense(&self, game: &Game)->DefensePower{
        DefensePower::max(
            self.role(game).defense(),
            PlayerComponent::<FragileVests>::get_defense_from_items(game, *self)
        )
    }
    pub fn increase_defense_to(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, defense: DefensePower){
        if defense.is_stronger(self.night_defense(game, midnight_variables)) {
            self.set_night_upgraded_defense(midnight_variables, Some(defense));
        }
    }

    pub fn push_night_messages_to_player(&self, game: &mut Game, midnight_variables: &mut MidnightVariables){
        let mut messages = self.night_messages(midnight_variables).to_vec();
        messages.shuffle(&mut rand::rng());
        messages.sort();
        self.send_packet(game, ToClientPacket::NightMessages { chat_messages: 
            messages.iter().map(|msg|ChatMessage::new_private(msg.clone())).collect()
        });
        self.add_private_chat_messages(game, messages);
    }

    pub fn chosen_vote(&self, game: &Game) -> Option<PlayerReference> {
        if let Some(PlayerListSelection(players)) = ControllerID::nominate(*self).get_player_list_selection(game) {
            Some(players.first().copied()).flatten()
        }else{
            None
        }
    }

    pub fn ability_deactivated_from_death(&self, game: &Game) -> bool {
        !(
            self.alive(game) ||
            (
                PlayerReference::all_players(game).any(|p|
                    if let RoleState::Medium(Medium{seanced_target: Some(player), ..}) = p.role_state(game) {
                        *player == *self
                    }else{
                        false
                    }
                )
            )
        )
    }
    

    /*
        Role functions
    */

    
    pub fn on_midnight_one_player(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, _priority: OnMidnightPriority) {
        if self.is_disconnected(game) && self.alive(game) {
            midnight_variables.get_mut(*self).died = true;
            midnight_variables.get_mut(*self).grave_killers = vec![GraveKiller::Quit]
        }
    }
    pub fn convert_selection_to_visits(&self, game: &Game) -> Vec<Visit> {
        self.role_state(game).clone().convert_selection_to_visits(game, *self)
    }
}



