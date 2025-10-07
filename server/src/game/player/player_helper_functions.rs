use rand::seq::SliceRandom;
use crate::{
    game::{
        attack_power::{AttackPower, DefensePower}, chat::{ChatMessage, ChatMessageVariant}, components::{
            attack::night_attack::NightAttack, fragile_vest::FragileVests, graves::{grave::{Grave, GraveKiller}, Graves}, insider_group::InsiderGroupID, night_visits::{NightVisitsIterator, Visits}, player_component::PlayerComponent, win_condition::WinCondition
        },
        controllers::{
            BooleanSelection, Controller, ControllerID, ControllerSelection, Controllers, PlayerListSelection, TwoPlayerOptionSelection
        },
        event::{
            on_any_death::OnAnyDeath, on_midnight::{MidnightVariables, OnMidnightPriority},
            on_player_roleblocked::OnPlayerRoleblocked, on_visit_wardblocked::OnVisitWardblocked, Event
        },
        game_conclusion::GameConclusion,
        role::{chronokaiser::Chronokaiser, medium::Medium, Role, RoleState},
        visit::{Visit, VisitTag},
        Game
    },
    packet::ToClientPacket, vec_set::VecSet
};

use super::PlayerReference;

impl PlayerReference{
    pub fn roleblock(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, send_messages: bool) {
        OnPlayerRoleblocked::new(*self, !send_messages).invoke(game, midnight_variables);
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

    /**
    ### Example use in witch case
        
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) {
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
                for (id, controller) in game.controllers.all_controllers().clone().iter() {
                    PlayerReference::possess_controller(game, id.clone(), controller, possessed_visit.target, possessed_into_visit.target)
                }

                possessed_visit.target.set_night_visits(midnight_variables,
                    possessed_visit.target.convert_selection_to_visits(game)
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
    fn possess_controller(game: &mut Game, id: ControllerID, controller: &Controller, possessed: PlayerReference, possessed_into: PlayerReference){
        match controller.selection() {
            ControllerSelection::Boolean(..) => {
                if possessed == possessed_into {
                    Controllers::set_selection_in_controller(
                        game,
                        Some(possessed),
                        id,
                        BooleanSelection(true),
                        true
                    );
                }
            },
            ControllerSelection::TwoPlayerOption(selection) => {

                let mut selection = selection.0;
                if let Some((_, second)) = selection {
                    selection = Some((possessed_into, second));
                }

                Controllers::set_selection_in_controller(
                    game,
                    Some(possessed),
                    id,
                    TwoPlayerOptionSelection(selection),
                    true
                );
            },
            ControllerSelection::PlayerList(selection) => {

                let mut selection = selection.0.clone();
                if let Some(first) = selection.first_mut(){
                    *first = possessed_into;
                }else{
                    selection = vec![possessed_into];
                }


                Controllers::set_selection_in_controller(
                    game,
                    Some(possessed),
                    id,
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
    pub fn possession_immune(&self, game: &Game) -> bool {
        self.role(game).possession_immune()
    }

    pub fn ward_night_action(&self, game: &mut Game, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority) -> Vec<PlayerReference> {
        match priority {
            OnMidnightPriority::PreWard => self.pre_ward(game, midnight_variables),
            OnMidnightPriority::Ward => self.ward(game, midnight_variables),
            _ => vec![]
        }
    }

    fn pre_ward(&self, game: &mut Game, midnight_variables: &mut MidnightVariables) -> Vec<PlayerReference> {
        let mut out = Vec::new();
        for visit in Visits::into_iter(midnight_variables) {
            if visit.wardblock_immune {
                continue;
            }
            if !matches!(visit.tag,
                VisitTag::Role { role: Role::Transporter, .. } |
                VisitTag::Role { role: Role::Warper, .. } |
                VisitTag::Role { role: Role::Porter, .. } |
                VisitTag::Role { role: Role::Polymath, id: 3 } |

                VisitTag::Role { role: Role::Witch, .. } |
                VisitTag::Role { role: Role::Retributionist, .. } |
                VisitTag::Role { role: Role::Necromancer, .. }
            ) {
                continue;
            }
            if visit.target != *self {continue;}
            OnVisitWardblocked::new(visit).invoke(game, midnight_variables);
            out.push(visit.visitor);
        }
        out
    }
    fn ward(&self, game: &mut Game, midnight_variables: &mut MidnightVariables) -> Vec<PlayerReference> {
        let mut out = Vec::new();
        for visit in Visits::into_iter(midnight_variables) {
            if visit.wardblock_immune {
                continue;
            }
            if visit.target != *self {continue;}
            OnVisitWardblocked::new(visit).invoke(game, midnight_variables);
            out.push(visit.visitor);
        }
        out
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



