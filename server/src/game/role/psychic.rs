use rand::seq::SliceRandom;
use serde::Serialize;
use crate::game::prelude::*;

#[derive(Debug, Clone, Serialize, Default)]
pub struct Psychic;


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

impl RoleStateTrait for Psychic {
    type ClientAbilityState = Psychic;
    fn on_midnight(self, game: &mut Game, _id: &AbilityID, actor_ref: PlayerReference, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if priority != OnMidnightPriority::Investigative {return}
        let Some(target_ref) = Visits::default_target(midnight_variables, actor_ref, Role::Psychic) else {return};

        actor_ref.push_night_message(midnight_variables, 
            if game.day_number() % 2 == 1 {
                Psychic::get_result_evil(game, midnight_variables, actor_ref, target_ref, Confused::is_confused(game, actor_ref))
            }else{
                Psychic::get_result_good(game, midnight_variables, actor_ref, target_ref, Confused::is_confused(game, actor_ref))
            }
        );
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Psychic, 0))
            .single_player_selection_typical(actor_ref, false, true)
            .night_typical(actor_ref)
            
            .build_map()
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        crate::game::role::common_role::convert_controller_selection_to_visits(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Psychic, 0),
            false
        )
    }
}

impl Psychic {
    fn get_result_evil(game: &mut Game, midnight_variables: &OnMidnightFold, actor_ref: PlayerReference, target: PlayerReference, confused: bool)->ChatMessageVariant{
        
        let mut valid_players: Vec<_> = Self::get_valid_players(game, actor_ref, target)
            .into_iter()
            .filter(|p|!Aura::innocent(game, midnight_variables, *p))
            .collect();

        valid_players.shuffle(&mut game.rng);

        #[expect(clippy::indexing_slicing, reason = "We're iterating over indexes, so it's safe")]
        for i in 0..valid_players.len(){
            #[expect(clippy::arithmetic_side_effects, reason = "`i` must be less than the list length, which must fit in usize.")]
            for j in i+1..valid_players.len(){
                if confused || Self::contains_evil(game, target, valid_players[i], valid_players[j]){
                    return ChatMessageVariant::PsychicEvil { first: valid_players[i], second: valid_players[j] }
                }
            }
        }

        ChatMessageVariant::PsychicFailed
    }
    fn get_result_good(game: &mut Game, midnight_variables: &OnMidnightFold, actor_ref: PlayerReference, target: PlayerReference, confused: bool)->ChatMessageVariant{
        let mut valid_players: Vec<_> = Self::get_valid_players(game, actor_ref, target)
            .into_iter()
            .filter(|p|!Aura::suspicious(game, midnight_variables, *p))
            .collect();

        valid_players.shuffle(&mut game.rng);

        for player in valid_players{
            if confused || Self::contains_good(game, target, player){
                return ChatMessageVariant::PsychicGood { player }
            }
        }

        ChatMessageVariant::PsychicFailed
    }

    fn player_is_evil(game: &Game, player_ref: PlayerReference)-> bool {
        !player_ref.win_condition(game).is_loyalist_for(GameConclusion::Town)
    }
    fn get_valid_players(game: &Game, actor_ref: PlayerReference, target: PlayerReference)->Vec<PlayerReference>{
        PlayerReference::all_players(game)
            .filter(|p|*p != actor_ref)
            .filter(|p|*p != target)
            .filter(|p|p.alive(game))
            .collect()
    }

    fn contains_evil(game: &Game, target: PlayerReference, a: PlayerReference, b: PlayerReference)->bool{
        [target, a, b].into_iter().any(|player_ref|Psychic::player_is_evil(game, player_ref))
    }
    fn contains_good(game: &Game, target: PlayerReference, player: PlayerReference)->bool{
        [target, player].into_iter().any(|player_ref|!Psychic::player_is_evil(game, player_ref))
    }
}