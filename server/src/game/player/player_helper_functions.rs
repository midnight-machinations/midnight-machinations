use std::collections::HashSet;
use rand::seq::SliceRandom;

use crate::{
    game::{
        attack_power::{AttackPower, DefensePower}, chat::{ChatGroup, ChatMessage, ChatMessageVariant}, components::{
            fragile_vest::FragileVests, graves::{grave::{Grave, GraveKiller}, Graves}, insider_group::InsiderGroupID, night_visits::{NightVisitsIterator, Visits}, player_component::PlayerComponent, win_condition::WinCondition
        }, controllers::{BooleanSelection, ControllerID, ControllerParametersMap, ControllerSelection, Controllers, PlayerListSelection, TwoPlayerOptionSelection}, event::{
            before_role_switch::BeforeRoleSwitch, on_any_death::OnAnyDeath,
            on_midnight::{MidnightVariables, OnMidnightPriority},
            on_player_roleblocked::OnPlayerRoleblocked, on_role_switch::OnRoleSwitch,
            on_visit_wardblocked::OnVisitWardblocked
        },
        game_conclusion::GameConclusion,
        modifiers::ModifierID, phase::PhaseType,
        role::{chronokaiser::Chronokaiser, medium::Medium, Role, RoleState},
        visit::{Visit, VisitTag},
        Game
    },
    packet::ToClientPacket, vec_map::VecMap, vec_set::VecSet
};

use super::PlayerReference;

impl PlayerReference{
    pub fn roleblock(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, send_messages: bool) {
        OnPlayerRoleblocked::new(*self, !send_messages).invoke(game, midnight_variables);
    }
    pub fn ward(&self, game: &mut Game, midnight_variables: &mut MidnightVariables) -> Vec<PlayerReference> {
        let mut out = Vec::new();
        for visit in Visits::into_iter(midnight_variables) {
            if !visit.wardblockable {
                continue;
            }
            if visit.target != *self {continue;}
            OnVisitWardblocked::new(visit).invoke(game, midnight_variables);
            out.push(visit.visitor);
        }
        out
    }

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
            &vec![attacker_ref].into_iter().collect(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            should_leave_death_note
        )
    }
    pub fn try_night_kill(&self, attacker_refs: &VecSet<PlayerReference>, game: &mut Game, midnight_variables: &mut MidnightVariables, grave_killer: GraveKiller, attack: AttackPower, should_leave_death_note: bool) -> bool {
        self.set_night_attacked(midnight_variables, true);

        if self.night_defense(game, midnight_variables).can_block(attack){
            self.push_night_message(midnight_variables, ChatMessageVariant::YouSurvivedAttack);
            for attacker in attacker_refs.iter() {
                attacker.push_night_message(midnight_variables, ChatMessageVariant::SomeoneSurvivedYourAttack);
            }
            return false;
        }
        
        self.push_night_message(midnight_variables, ChatMessageVariant::YouWereAttacked);
        for attacker in attacker_refs.iter() {
            attacker.push_night_message(midnight_variables, ChatMessageVariant::YouAttackedSomeone);
        }

        self.push_night_grave_killers(midnight_variables, grave_killer);
            
        if should_leave_death_note {
            for attacker in attacker_refs.iter() {
                if let Some(note) = attacker.death_note(game) {
                    self.push_night_grave_death_notes(midnight_variables, note.clone());
                }
            }
        }
        

        if !self.alive(game) { return true }

        self.set_night_died(midnight_variables, true);

        true
    }
    pub fn try_night_kill_no_attacker(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, grave_killer: GraveKiller, attack: AttackPower) -> bool {
        self.try_night_kill(
            &VecSet::new(),
            game,
            midnight_variables,
            grave_killer,
            attack,
            false
        )
    }

    /**
    ### Example use in witch case
        
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        if let Some(currently_used_player) = actor_ref.possess_night_action(game, self.currently_used_player){
            actor_ref.set_role_state(game, RoleState::Witch(Witch{
                currently_used_player: Some(currently_used_player)
            }))
        }
    }
    */
    pub fn possess_night_action(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority, currently_used_player: Option<PlayerReference>)->Option<PlayerReference>{
        match priority {
            OnMidnightPriority::Possess => {
                let untagged_possessor_visits = self.untagged_night_visits_cloned(midnight_variables);
                let possessed_visit = untagged_possessor_visits.get(0)?;
                let possessed_into_visit = untagged_possessor_visits.get(1)?;
                
                possessed_visit.target.push_night_message(midnight_variables,
                    ChatMessageVariant::YouWerePossessed { immune: possessed_visit.target.possession_immune(game) }
                );
                if possessed_visit.target.possession_immune(game) {
                    self.push_night_message(midnight_variables,
                        ChatMessageVariant::TargetIsPossessionImmune
                    );
                    return None;
                }


                //change all controller inputs to be selecting this player as well
                for (controller_id, controller_data) in game.controllers.all_controllers().clone().iter() {
                    match controller_data.selection() {
                        ControllerSelection::Boolean(..) => {
                            if possessed_visit.target == possessed_into_visit.target {
                                Controllers::set_selection_in_controller(
                                    game,
                                    possessed_visit.target,
                                    controller_id.clone(),
                                    BooleanSelection(true),
                                    true
                                );
                            }
                        },
                        ControllerSelection::TwoPlayerOption(selection) => {

                            let mut selection = selection.0;
                            if let Some((_, second)) = selection {
                                selection = Some((possessed_into_visit.target, second));
                            }

                            Controllers::set_selection_in_controller(
                                game,
                                possessed_visit.target,
                                controller_id.clone(),
                                TwoPlayerOptionSelection(selection),
                                true
                            );
                        },
                        ControllerSelection::PlayerList(selection) => {

                            let mut selection = selection.0.clone();
                            if let Some(first) = selection.first_mut(){
                                *first = possessed_into_visit.target;
                            }else{
                                selection = vec![possessed_into_visit.target];
                            }


                            Controllers::set_selection_in_controller(
                                game,
                                possessed_visit.target,
                                controller_id.clone(),
                                PlayerListSelection(selection),
                                true
                            );
                        },
                        ControllerSelection::Unit(..) |
                        ControllerSelection::ChatMessage(..) |
                        ControllerSelection::RoleList(..) |
                        ControllerSelection::TwoRoleOption(..) |
                        ControllerSelection::TwoRoleOutlineOption(..) |
                        ControllerSelection::String(..) |
                        ControllerSelection::Integer(..) |
                        ControllerSelection::Kira(..) => {}
                    }
                }

                possessed_visit.target.set_night_visits(midnight_variables,
                    possessed_visit.target.convert_selection_to_visits(game)
                );

                //remove the second role visit from the possessor
                self.set_night_visits(
                    midnight_variables,
                    self.all_night_visits_cloned(midnight_variables)
                        .into_iter()
                        .filter(|v|v.tag != VisitTag::Role { role: self.role(game), id: 1 })
                        .collect()
                );
                Some(possessed_visit.target)
            },
            OnMidnightPriority::Investigative => {
                if let Some(currently_used_player) = currently_used_player {
                    self.push_night_message(midnight_variables,
                        ChatMessageVariant::TargetHasRole { role: currently_used_player.role(game) }
                    );
                }
                None
            },
            OnMidnightPriority::StealMessages => {
                if let Some(currently_used_player) = currently_used_player {
                    for message in currently_used_player.night_messages(midnight_variables).clone() {
                        self.push_night_message(midnight_variables,
                            ChatMessageVariant::TargetsMessage { message: Box::new(message.clone()) }
                        );
                    }
                }
                None
            },
            _ => {
                None
            }
        }
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
        let new_role_data = self.role(game).new_state(game);
        self.set_role_state(game, new_role_data.clone());
        self.on_role_creation(game);    //this function can change role state
        if new_role_data.role() == self.role(game) {
            self.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role: self.role(game)});
        }
        self.set_win_condition(game, self.win_condition(game).clone());
        InsiderGroupID::set_player_insider_groups(
            InsiderGroupID::all_groups_with_player(game, *self), 
            game, *self
        );
    }
    pub fn set_role(&self, game: &mut Game, new_role_data: impl Into<RoleState>) {
        let new_role_data = new_role_data.into();
        let old = self.role_state(game).clone();
        BeforeRoleSwitch::new(*self, old.clone(), new_role_data.clone()).invoke(game);

        self.set_role_state(game, new_role_data.clone());
        self.on_role_creation(game);    //this function can change role state
        if new_role_data.role() == self.role(game) {
            self.add_private_chat_message(game, ChatMessageVariant::RoleAssignment{role: self.role(game)});
        }

        OnRoleSwitch::new(*self, old, self.role_state(game).clone()).invoke(game);
    }
    /// Swaps this persons role, sends them the role chat message, and makes associated changes
    pub fn set_role_and_win_condition_and_revealed_group(&self, game: &mut Game, new_role_data: impl Into<RoleState>){
        let new_role_data = new_role_data.into();
        
        self.set_role(game, new_role_data);
    
        self.set_win_condition(game, self.role_state(game).clone().default_win_condition());
        
        InsiderGroupID::set_player_insider_groups(
            self.role_state(game).clone().default_revealed_groups(), 
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
    
    
    pub fn tracker_seen_visits(self, _game: &Game, midnight_variables: &MidnightVariables) -> Vec<Visit> {
        if let Some(v) = self.night_appeared_visits(midnight_variables) {
            v.clone()
        } else {
            self.all_night_visits_cloned(midnight_variables)
        }
    }
    pub fn all_appeared_visitors(self, game: &Game, midnight_variables: &MidnightVariables) -> Vec<PlayerReference> {
        PlayerReference::all_players(game).filter(|player_ref|{
            player_ref.tracker_seen_visits(game, midnight_variables).iter().any(|other_visit| 
                other_visit.target == self
            )
        }).collect()
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

    pub fn revealed_players_map(&self, game: &Game) -> VecMap<PlayerReference, Role> {
        let mut map = VecMap::new();
        for player in self.revealed_players(game).iter() {
            map.insert(*player, player.role(game));
        }
        map
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
    pub fn possession_immune(&self, game: &Game) -> bool {
        self.role(game).possession_immune()
    }
    pub fn get_won_game(&self, game: &Game, conclusion: GameConclusion) -> bool {
        match self.win_condition(game){
            WinCondition::GameConclusionReached { win_if_any } => win_if_any.contains(&conclusion),
            WinCondition::RoleStateWon => {
                match self.role_state(game) {
                    RoleState::Jester(r) => r.won(),
                    RoleState::Doomsayer(r) => r.won(),
                    RoleState::Mercenary(r) => r.won(),
                    RoleState::Revolutionary(r) => r.won(),
                    RoleState::Chronokaiser(_) => Chronokaiser::won(game, *self),
                    RoleState::Martyr(r) => r.won(),
                    _ => false
                }
            },
        }
    }
    /// If they can consistently kill then they keep the game running
    /// Town kills by voting
    /// Mafia kills with MK or gun
    /// Cult kills / converts
    pub fn keeps_game_running(&self, game: &Game) -> bool {
        if InsiderGroupID::Mafia.contains_player(game, *self) {return true;}
        if InsiderGroupID::Cult.contains_player(game, *self) {return true;}
        if self.win_condition(game).is_loyalist_for(GameConclusion::Town) {return true;}
        
        GameConclusion::keeps_game_running(self.role(game))
    }

    /*
        Role functions
    */

    pub fn controller_parameters_map(&self, game: &Game) -> ControllerParametersMap {
        self.role_state(game).clone().controller_parameters_map(game, *self)
    }
    pub fn on_midnight_one_player(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
        self.role_state(game).clone().on_midnight(game, midnight_variables, *self, priority);

        if self.is_disconnected(game) && self.alive(game) {
            midnight_variables.get_mut(*self).died = true;
            midnight_variables.get_mut(*self).grave_killers = vec![GraveKiller::Quit]
        }
    }
    pub fn on_conceal_role(&self, game: &mut Game, player: PlayerReference, concealed_player: PlayerReference) {
        self.role_state(game).clone().on_conceal_role(game, *self, player, concealed_player)
    }
    pub fn on_role_creation(&self, game: &mut Game) {
        self.role_state(game).clone().on_role_creation(game, *self)
    }
    pub fn get_current_send_chat_groups(&self, game: &Game) -> HashSet<ChatGroup> {
        if game.modifier_settings().is_enabled(ModifierID::NoChat)
            || (
                game.modifier_settings().is_enabled(ModifierID::NoNightChat)
                && self.alive(game)
                && matches!(game.current_phase().phase(), PhaseType::Night | PhaseType::Obituary)
            )
        {
            return HashSet::new()
        }
        self.role_state(game).clone().get_current_send_chat_groups(game, *self)
    }
    pub fn get_current_receive_chat_groups(&self, game: &Game) -> HashSet<ChatGroup> {
        self.role_state(game).clone().get_current_receive_chat_groups(game, *self)
    }
    pub fn convert_selection_to_visits(&self, game: &Game) -> Vec<Visit> {
        self.role_state(game).clone().convert_selection_to_visits(game, *self)
    }
}



