use crate::game::{
    Game, chat::ChatMessageVariant, components::night_visits::{NightVisitsIterator, Visits, visit::VisitTag}, controllers::{
        BooleanSelection, ControllerID, ControllerSelection, Controllers, PlayerListSelection,
        TwoPlayerOptionSelection
    }, event::{Invokable as _, on_midnight::{OnMidnightFold, OnMidnightPriority}, on_player_possessed::OnPlayerPossessed},
    player::PlayerReference, role::Role
};

pub struct Possession;
impl Possession {
    pub fn possession_immune(id: &ControllerID)->bool{
        match id {
            ControllerID::Role { role: Role::Lich, id: 7, .. } => true,
            ControllerID::Role { role, .. } => {
                matches!(role, 
                    Role::Veteran | Role::Medium
                    | Role::Bouncer | Role::Scarecrow
                    | Role::Transporter  | Role::Warper | Role::Porter
                    | Role::Witch
                    | Role::Dreamcatcher
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
    pub fn possess_night_action(
        game: &mut Game,
        fold: &mut OnMidnightFold,

        tag_possess: VisitTag,
        tag_possess_into: VisitTag,
    ) -> Option<PlayerReference> {

        let possessed = Visits::into_iter(fold)
            .with_tag(tag_possess)
            .map_target()
            .next()?;

        let into = Visits::into_iter(fold)
            .with_tag(tag_possess_into)
            .map_target()
            .next()?;
        
        possessed.push_night_message(fold, ChatMessageVariant::YouWerePossessed);
        (
            &OnPlayerPossessed::new(
                possessed,
                into
            ),
            fold
        ).invoke(game);

        Some(possessed)
    }
    pub fn possess_night_action_and_steal_messages(
        actor: PlayerReference,
        game: &mut Game,
        fold: &mut OnMidnightFold,
        priority: OnMidnightPriority,

        currently_used_player: Option<PlayerReference>,

        tag_possess: VisitTag,
        tag_possess_into: VisitTag,
    )->Option<PlayerReference>{
        match priority {
            OnMidnightPriority::Possess => {
                Self::possess_night_action(game, fold, tag_possess, tag_possess_into)
            },
            OnMidnightPriority::Investigative => {
                if let Some(currently_used_player) = currently_used_player {
                    actor.push_night_message(fold,
                        ChatMessageVariant::TargetHasRole { role: currently_used_player.role(game) }
                    );
                }
                None
            },
            OnMidnightPriority::StealMessages => {
                if let Some(currently_used_player) = currently_used_player {
                    for message in currently_used_player.night_messages(fold).clone() {
                        actor.push_night_message(fold,
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
                        false
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
                    false
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
                    false
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