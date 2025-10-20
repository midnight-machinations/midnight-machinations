use serde::Serialize;
use crate::game::prelude::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hypnotist{
    pub roleblock: bool,
    pub you_were_roleblocked_message: bool,
    pub you_survived_attack_message: bool,
    pub you_were_guarded_message: bool,
    pub you_were_transported_message: bool,
    pub you_were_possessed_message: bool,
    pub you_were_wardblocked_message: bool,
}


impl Default for Hypnotist {
    fn default() -> Self {
        Self {
            roleblock: true,
            you_were_roleblocked_message: true,
            you_survived_attack_message: false,
            you_were_guarded_message: false,
            you_were_transported_message: false,
            you_were_possessed_message: false,
            you_were_wardblocked_message: false,
        }
    }
}

pub(super) const MAXIMUM_COUNT: Option<u8> = Some(1);
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Hypnotist {
    type ClientAbilityState = Hypnotist;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Hypnotist) else {return};

        match priority {
            OnMidnightPriority::TopPriority => {
                let mut hypnotist = self.clone();
                hypnotist.ensure_at_least_one_message();
                actor_ref.edit_role_ability_helper(game, RoleState::Hypnotist(self));
            },
            OnMidnightPriority::Roleblock => {
                if self.roleblock {
                    target_ref.roleblock(game, midnight_variables, false);
                }
            },
            OnMidnightPriority::Deception => {
                if actor_ref.night_blocked(midnight_variables) {
                    if self.roleblock {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::RoleBlocked);
                    }
                }else{
                    if self.you_were_roleblocked_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::RoleBlocked);
                    }
                    if self.you_survived_attack_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::YouSurvivedAttack);
                    }
                    if self.you_were_guarded_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::YouWereGuarded);
                    }
                    if self.you_were_transported_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::Transported);
                    }
                    if self.you_were_possessed_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::YouWerePossessed);
                    }
                    if self.you_were_wardblocked_message {
                        target_ref.push_night_message(midnight_variables, ChatMessageVariant::Wardblocked);
                    }
                }
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Hypnotist, 0))
            .single_player_selection_typical(actor_ref, false, false)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Hypnotist, 0),
            false
        )
    }
     fn default_revealed_groups(self) -> crate::vec_set::VecSet<crate::game::components::insider_group::InsiderGroupID> {
        vec![
            crate::game::components::insider_group::InsiderGroupID::Mafia
        ].into_iter().collect()
    }
    fn on_player_roleblocked(self, _game: &mut Game, _midnight_variables: &mut OnMidnightFold, _actor_ref: PlayerReference, _player: PlayerReference, _invisible: bool) {}
}
impl Hypnotist {
    pub fn ensure_at_least_one_message(&mut self){
        if
            !self.you_were_roleblocked_message && 
            !self.you_survived_attack_message && 
            !self.you_were_guarded_message && 
            !self.you_were_transported_message && 
            !self.you_were_possessed_message && 
            !self.you_were_wardblocked_message
        {
            self.you_were_roleblocked_message = true;
        }
    }
}