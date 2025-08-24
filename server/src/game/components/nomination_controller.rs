use crate::game::{
        controllers::{
            AvailablePlayerListSelection, ControllerID,
            ControllerParametersMap, PlayerListSelection
        }, chat::{ChatGroup, ChatMessageVariant}, event::on_validated_ability_input_received::OnValidatedControllerInputReceived, modifiers::{hidden_nomination_votes::HiddenNominationVotes, ModifierType, Modifiers}, player::PlayerReference, Game
    };

use super::forfeit_vote::ForfeitNominationVote;

pub struct NominationController;

impl NominationController{
    pub fn controller_parameters_map(game: &mut Game)->ControllerParametersMap{
        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|actor| Self::one_player_controller(game, actor))
        )
    }
    fn one_player_controller(game: &mut Game, actor: PlayerReference)->ControllerParametersMap{
        ControllerParametersMap::builder(game)
            .id(crate::game::controllers::ControllerID::Nominate { player: actor })
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game).filter(|p|p.alive(game)).collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .add_grayed_out_condition(
                !actor.alive(game) ||
                ForfeitNominationVote::forfeited_vote(game, actor) ||
                game.current_phase().phase() != crate::game::phase::PhaseType::Nomination
            )
            .reset_on_phase_start(crate::game::phase::PhaseType::Nomination)
            .allow_players([actor])
            .build_map()
    }
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()){
        if let Some(PlayerListSelection(voted)) = event.input.get_player_list_selection_if_id(ControllerID::Nominate{ player: event.actor_ref }){

            if !HiddenNominationVotes::nomination_votes_are_hidden(game) {
                game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::Voted{
                    voter: event.actor_ref.index(), 
                    votee: voted.first().map(|p|p.index())
                });
            }

            game.count_nomination_and_start_trial(
                Modifiers::is_enabled(game, ModifierType::UnscheduledNominations)
            );

            game.send_player_votes();
        }
    }
}