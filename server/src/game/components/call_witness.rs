use std::{collections::HashSet, iter::once};

use crate::{
    game::{
        chat::{ChatGroup, ChatMessageVariant, PlayerChatGroupMap}, components::silenced::Silenced,
        controllers::{AvailablePlayerListSelection, ControllerID, ControllerParametersMap, PlayerListSelection},
        event::{on_phase_start::OnPhaseStart, on_validated_ability_input_received::OnValidatedControllerInputReceived}, phase::{PhaseState, PhaseType}, player::PlayerReference, role::RoleState, Game
    },
    vec_set::VecSet
};


pub struct CallWitness;

impl CallWitness{
    pub fn send_player_chat_group_map(game: &Game) -> PlayerChatGroupMap {
        let mut out = PlayerChatGroupMap::new();
        for player in Self::witness_called(game){
            if Silenced::silenced(game, player) {continue;}
            out.insert(player, ChatGroup::All);
        }
        out
    }
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

        if allowed_players.is_empty() && !Silenced::silenced(game, actor) {
            allowed_players.insert(actor);
        }

        ControllerParametersMap::builder(game)
            .id(crate::game::controllers::ControllerID::CallWitness { player: actor })
            .available_selection(AvailablePlayerListSelection {
                available_players: PlayerReference::all_players(game)
                    .filter(|p|p.alive(game))
                    .filter(|p|*p != actor)
                    .collect(),
                can_choose_duplicates: false,
                max_players: None
            })
            .add_grayed_out_condition(!matches!(game.current_phase().phase(), PhaseType::Nomination|PhaseType::Testimony))
            .reset_on_phase_start(crate::game::phase::PhaseType::Judgement)
            .allow_players(allowed_players)
            .build_map()
    }
    pub fn witness_called(game: &Game)->HashSet<PlayerReference>{
        if
            let PhaseState::Testimony{player_on_trial, ..} = game.phase_machine.current_state &&
            let Some(PlayerListSelection(players)) = (ControllerID::CallWitness { player: player_on_trial }).get_player_list_selection(game)
        {
            players.clone().into_iter().chain(once(player_on_trial)).collect()
        }else{
            HashSet::new()
        }
    }
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()){
        let PhaseState::Testimony { player_on_trial, .. } = game.current_phase() else {return};
        let ControllerID::CallWitness { player: player_controller_changed, .. } = event.input.id() else {return};

        if *player_on_trial != player_controller_changed {return};

        Self::send_witness_called_message(game)
    }
    pub fn on_phase_start(game: &mut Game, _event: &OnPhaseStart, _fold: &mut (), _priority: ()){
        Self::send_witness_called_message(game)
    }

    fn send_witness_called_message(game: &mut Game){
        let PhaseState::Testimony { player_on_trial, .. } = game.current_phase() else {return};

        let Some(PlayerListSelection(witnesses)) = ControllerID::CallWitness { player: *player_on_trial }
            .get_player_list_selection(game)
            .cloned()
            else {return};

        game.add_message_to_chat_group(ChatGroup::All, ChatMessageVariant::WitnessesCalled{
            player_on_trial: *player_on_trial,
            witnesses
        });
    }
}