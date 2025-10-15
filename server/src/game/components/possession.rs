use crate::game::{
    chat::ChatMessageVariant, 
    controllers::{
        BooleanSelection, ControllerID, ControllerSelection, Controllers, PlayerListSelection,
        TwoPlayerOptionSelection
    },
    event::{on_midnight::{MidnightVariables, OnMidnightPriority},
    on_player_possessed::OnPlayerPossessed},
    player::PlayerReference, role::Role, Game
};

pub struct Possession;
impl Possession {
    pub fn possession_immune(id: &ControllerID)->bool{
        match id {
            ControllerID::Role { role, .. } => {
                matches!(role, 
                    Role::Veteran | Role::Medium
                    | Role::Bouncer | Role::Scarecrow
                    | Role::Transporter  | Role::Warper | Role::Porter
                    | Role::Necromancer | Role::Retributionist | Role::Witch
                )
            },
            ControllerID::SyndicateGunShoot => false,
            ControllerID::SyndicateBackupAttack => false,
            _ => true
        }
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
    pub fn possess_night_action(possessor: PlayerReference, game: &mut Game, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority, currently_used_player: Option<PlayerReference>)->Option<PlayerReference>{
        match priority {
            OnMidnightPriority::Possess => {
                let untagged_possessor_visits = possessor.role_night_visits_cloned(midnight_variables);
                let possessed_visit = untagged_possessor_visits.get(0)?;
                let possessed_into_visit = untagged_possessor_visits.get(1)?;
                
                possessed_visit.target.push_night_message(midnight_variables, ChatMessageVariant::YouWerePossessed);
                OnPlayerPossessed::new(
                    possessed_visit.target,
                    possessed_into_visit.target
                ).invoke(game, midnight_variables);

                Some(possessed_visit.target)
            },
            OnMidnightPriority::Investigative => {
                if let Some(currently_used_player) = currently_used_player {
                    possessor.push_night_message(midnight_variables,
                        ChatMessageVariant::TargetHasRole { role: currently_used_player.role(game) }
                    );
                }
                None
            },
            OnMidnightPriority::StealMessages => {
                if let Some(currently_used_player) = currently_used_player {
                    for message in currently_used_player.night_messages(midnight_variables).clone() {
                        possessor.push_night_message(midnight_variables,
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
    pub fn possess_controller(game: &mut Game, id: ControllerID, possessed: PlayerReference, possessed_into: PlayerReference){
        let Some(controller) = game.controllers.controllers.get(&id) else {return};
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
}