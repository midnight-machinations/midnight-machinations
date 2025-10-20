use serde::Serialize;
use crate::game::prelude::*;
use crate::game::role::detective::Detective;
use crate::vec_set::VecSet;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct TallyClerk;


impl RoleStateTrait for TallyClerk {
    type ClientAbilityState = TallyClerk;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}
        let target = 
        if 
            let Some(target) = Visits::default_target(midnight_variables, actor_ref, Role::TallyClerk) &&
            VerdictsToday::player_was_on_trial(game, target)
        {
            Some(target)
        }else{
            None
        };

        let evil_count = if Confused::is_confused(game, actor_ref){
            Self::confused_result(game, midnight_variables, target)
        }else{
            Self::result(game, midnight_variables, target)
        };
        
        let message = ChatMessageVariant::TallyClerkResult{ evil_count };
        actor_ref.push_night_message(midnight_variables, message);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::TallyClerk, 0))
            .player_list_selection_typical(actor_ref, true, true, false, true, true, Some(1))
            .night_typical(actor_ref)
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::TallyClerk, 0),
            false
        )
    }
}

impl TallyClerk {
    fn get_guilties(game: &Game, target: Option<PlayerReference>)->VecSet<PlayerReference>{
        if let Some(target) = target {
            VerdictsToday::guilties_during_trial(game, target)
        }else{
            VerdictsToday::guilties_during_any_trial(game)
        }
    }
    fn result(game: &Game, midnight_variables: &OnMidnightFold, target: Option<PlayerReference>)->u8{
        let guilties = Self::get_guilties(game, target);
        PlayerReference::all_players(game)
            .filter(|player|guilties.contains(player))
            .filter(|player|TallyClerk::player_is_suspicious(game, midnight_variables, *player))
            .count()
            .try_into()
            .unwrap_or(u8::MAX)
    }
    fn confused_result(game: &Game, midnight_variables: &OnMidnightFold, target: Option<PlayerReference>)->u8{
        let guilties = Self::get_guilties(game, target);
        let total_guilties = guilties.count();

        let evil_count = Self::result(game, midnight_variables, target).saturating_add_signed(rand::random_range(0..=1));
        
        evil_count.min(total_guilties.try_into().unwrap_or(u8::MAX))
    }
    fn player_is_suspicious(game: &Game, midnight_variables: &OnMidnightFold, target: PlayerReference) -> bool {
        Detective::player_is_suspicious(game, midnight_variables, target)
    }
}