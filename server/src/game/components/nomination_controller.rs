use crate::{
    game::{
        ability_input::{AbilityInput, AvailablePlayerListSelection, ControllerID, ControllerParametersMap, PlayerListSelection}, chat::{ChatGroup, ChatMessageVariant}, modifiers::{ModifierType, Modifiers}, phase::PhaseState, player::PlayerReference, Game
    }, packet::ToClientPacket
};

use super::forfeit_vote::ForfeitVote;

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
            .id(crate::game::ability_input::ControllerID::Nominate { player: actor })
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game).filter(|p|p.alive(game)).collect(),
                can_choose_duplicates: false,
                max_players: Some(1)
            })
            .add_grayed_out_condition(
                !actor.alive(game) ||
                ForfeitVote::forfeited_vote(game, actor) ||
                game.current_phase().phase() != crate::game::phase::PhaseType::Nomination
            )
            .reset_on_phase_start_with_condition(crate::game::phase::PhaseType::Nomination, |game| {
                Modifiers::is_enabled(game, ModifierType::UnscheduledNominations) || {
                    if let PhaseState::Nomination { trials_left, .. } = game.current_phase() {
                        // Less than 3 trials left means last phase was also nomination.
                        // -- as long as Scheduled Nominations is enabled
                        *trials_left == 3
                    } else {
                        false
                    }
                }
            })
            .allow_players([actor])
            .build_map()
    }
    pub fn on_validated_ability_input_received(game: &mut Game, player: PlayerReference, input: AbilityInput){
        if let Some(PlayerListSelection(voted)) = input.get_player_list_selection_if_id(ControllerID::Nominate{ player }){

            game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::Voted{
                voter: player.index(), 
                votee: voted.first().map(|p|p.index())
            });

            game.count_nomination_and_start_trial(
                Modifiers::is_enabled(game, ModifierType::UnscheduledNominations)
            );

            game.send_packet_to_all(ToClientPacket::PlayerVotes {
                votes_for_player: game.create_voted_player_map()
            });
        }
    }
}