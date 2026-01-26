use serde::Serialize;
use crate::game::prelude::*;
use crate::game::role::detective::Detective;

pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
pub struct TallyClerk;


impl RoleStateTrait for TallyClerk {
    type ClientAbilityState = TallyClerk;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return;}

        let targets = Visits::into_iter(midnight_variables)
            .default_role_visits(actor_ref, Role::TallyClerk)
            .map_target()
            .flat_map(|my_target|
                Visits::into_iter(midnight_variables)
                    .with_visitor(my_target)
                    .with_appeared(midnight_variables)
                    .map_target()
            )
            .collect::<Box<[PlayerReference]>>();

        let evil_count = if Confused::is_confused(game, actor_ref){
            Self::confused_result(game, midnight_variables, &targets)
        }else{
            Self::result(game, midnight_variables, &targets)
        };
        
        let message = ChatMessageVariant::TallyClerkResult{ evil_count };
        actor_ref.push_night_message(midnight_variables, message);
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::TallyClerk, 0))
            .player_list_selection_typical(
                actor_ref,
                false,
                true,
                false,
                false,
                true,
                None
            )
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
    fn result(game: &Game, midnight_variables: &OnMidnightFold, targets: &[PlayerReference])->u8{
        targets
            .iter()
            .filter(|player|TallyClerk::player_is_suspicious(game, midnight_variables, **player))
            .count()
            .try_into()
            .unwrap_or(u8::MAX)
    }
    fn confused_result(game: &Game, midnight_variables: &OnMidnightFold, targets: &[PlayerReference])->u8{
        let total_players = targets.len();

        let evil_count = Self::result(game, midnight_variables, targets).saturating_add_signed(rand::random_range(0..=1));
        
        evil_count.min(total_players.try_into().unwrap_or(u8::MAX))
    }
    fn player_is_suspicious(game: &Game, midnight_variables: &OnMidnightFold, target: PlayerReference) -> bool {
        Detective::player_is_suspicious(game, midnight_variables, target)
    }
}