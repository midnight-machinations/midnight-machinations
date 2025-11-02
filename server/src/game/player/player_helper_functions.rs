use rand::seq::SliceRandom;
use crate::{
    game::{
        attack_power::DefensePower,
        chat::{ChatMessage, ChatMessageVariant}, components::{
            fragile_vest::FragileVests, graves::{grave::{Grave, GraveKiller}, Graves},
            insider_group::InsiderGroupID, player_component::PlayerComponent,
            role::RoleComponent,
        }, controllers::{ControllerID, PlayerListSelection}, event::{
            on_any_death::OnAnyDeath, on_midnight::{OnMidnightFold, OnMidnightPriority}, AsInvokable as _, Invokable as _,
        }, role::{medium::Medium, necromancer::Necromancer, RoleState}, Game
    },
    packet::ToClientPacket,
};

use super::PlayerReference;

impl PlayerReference{
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
        OnAnyDeath::new(*self).as_invokable().invoke(game);
    }
    pub fn initial_set_role_insider_wincondition(&self, game: &mut Game){
        self.set_win_condition(game, self.win_condition(game).clone());
        InsiderGroupID::set_player_insider_groups(
            InsiderGroupID::all_groups_with_player(game, *self), 
            game, *self
        );
        if self.chat_messages(game).iter().all(|m|*m.variant() != ChatMessageVariant::RoleAssignment { role: self.role(game) }) {
            RoleComponent::set_role_without_ability(*self, game, self.role(game));
        }
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
    pub fn set_role_win_con_insider_group_midnight(&self, game: &mut Game, midnight_variables: &mut OnMidnightFold, new_role_data: impl Into<RoleState>){
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
    pub fn increase_defense_to(&self, game: &mut Game, midnight_variables: &mut OnMidnightFold, defense: DefensePower){
        if defense.is_stronger(self.night_defense(game, midnight_variables)) {
            self.set_night_upgraded_defense(midnight_variables, Some(defense));
        }
    }

    pub fn push_night_messages_to_player(&self, game: &mut Game, midnight_variables: &mut OnMidnightFold){
        let mut messages = self.night_messages(midnight_variables).to_vec();
        messages.shuffle(&mut game.rng);
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
            PlayerReference::all_players(game)
                .any(|medium_player|
                    Medium::get_seanced_targets(game, medium_player).contains(self) ||
                    Necromancer::get_seanced_targets(game, medium_player).contains(self)
                )
            
        )
    }
    

    /*
        Role functions
    */

    
    pub fn on_midnight_one_player(&self, game: &mut Game, midnight_variables: &mut OnMidnightFold, _priority: OnMidnightPriority) {
        if self.is_disconnected(game) && self.alive(game) {
            midnight_variables.get_mut(*self).died = true;
            midnight_variables.get_mut(*self).grave_killers = vec![GraveKiller::Quit]
        }
    }
}



