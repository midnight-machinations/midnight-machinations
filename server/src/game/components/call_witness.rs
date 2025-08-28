use std::collections::HashSet;

use crate::{
    game::{
        controllers::{AvailablePlayerListSelection, ControllerID, ControllerParametersMap, PlayerListSelection}, event::on_validated_ability_input_received::OnValidatedControllerInputReceived, phase::PhaseState, player::PlayerReference, role::RoleState, Game
    }, packet::ToClientPacket, vec_set::VecSet
};


pub struct CallWitness;

impl CallWitness{
    pub fn controller_parameters_map(game: &mut Game)->ControllerParametersMap{
        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|actor| Self::one_player_controller(game, actor))
        )
    }
    fn one_player_controller(game: &mut Game, actor: PlayerReference)->ControllerParametersMap{
        let mut allowed_players: VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p|
                if let RoleState::Cerenovous(cerenovous) = p.role_state(game){
                    cerenovous.currently_brained == Some(actor)
                }else{false}
            )
            .collect();
        if allowed_players.is_empty() {
            allowed_players.insert(actor);
        }

        ControllerParametersMap::builder(game)
            .id(crate::game::controllers::ControllerID::CallWitness { player: actor })
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game).filter(|p|p.alive(game)).collect(),
                can_choose_duplicates: false,
                max_players: None
            })
            .reset_on_phase_start(crate::game::phase::PhaseType::Judgement)
            .default_selection(PlayerListSelection::one(Some(actor)))
            .allow_players(allowed_players)
            .build_map()
    }
    pub fn witness_called(game: &Game)->HashSet<PlayerReference>{
        if
            let PhaseState::Testimony{player_on_trial, ..} = game.phase_machine.current_state &&
            let Some(PlayerListSelection(players)) = (ControllerID::CallWitness { player: player_on_trial }).get_player_list_selection(game)
        {
            players.clone().into_iter().collect()
        }else{
            HashSet::new()
        }
    }
    pub fn on_validated_ability_input_received(game: &mut Game, _event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()){
        for player in PlayerReference::all_players(game){
            player.send_packet(game, ToClientPacket::YourSendChatGroups { send_chat_groups: 
                player.get_current_send_chat_groups(game).into_iter().collect()
            });
        }
    }
}