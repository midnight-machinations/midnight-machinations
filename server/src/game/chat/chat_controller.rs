

use crate::{
    game::{
        controllers::{
            AvailableBooleanSelection,
            AvailableStringSelection, AvailableUnitSelection, BooleanSelection,
            ControllerID, ControllerParametersMap, PlayerListSelection, StringSelection
        },
        chat::{ChatComponent, ChatGroup, ChatMessageVariant, MessageSender},
        event::{on_validated_ability_input_received::OnValidatedControllerInputReceived, on_whisper::OnWhisper, Event}, player::PlayerReference,
        role::{Role, RoleState}, Game
    },
    strings::TidyableString, vec_set::VecSet
};

impl ChatComponent{
    pub fn on_validated_ability_input_received(game: &mut Game, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()){
        match event.input.id() {
            ControllerID::SendChat { player } => {
                Self::send_chat(game, player);
            },
            ControllerID::SendWhisper { player } => {
                Self::send_whisper(game, player);
            }
            _ => {}
        }
    }


    fn send_chat(game: &mut Game, sender_player: PlayerReference){

        let Some(StringSelection(text)) = ControllerID::chat(sender_player).get_string_selection(game).cloned() else {return};
        let Some(BooleanSelection(block)) = ControllerID::chat_is_block(sender_player).get_boolean_selection(game).cloned() else {return};

        if text.replace(['\n', '\r'], "").trim().is_empty() {
            return;
        }
        
        for chat_group in sender_player.get_current_send_chat_groups(game){
            let message_sender = match chat_group {
                ChatGroup::Jail if sender_player.role(game) == Role::Jailor => {
                    Some(MessageSender::Jailor)
                },
                ChatGroup::Kidnapped if sender_player.role(game) == Role::Kidnapper => { 
                    Some(MessageSender::Jailor)
                },
                ChatGroup::Dead if sender_player.alive(game) => {
                    Some(MessageSender::LivingToDead{ player: sender_player })
                },
                ChatGroup::Interview if sender_player.role(game) == Role::Reporter => {
                    Some(MessageSender::Reporter)
                },
                _ => {None}
            };

            let message_sender = message_sender.unwrap_or(MessageSender::Player { player: sender_player });


            game.add_message_to_chat_group(
                chat_group.clone(),
                ChatMessageVariant::Normal{
                    message_sender,
                    text: text.trim_newline().trim_whitespace().truncate(600).truncate_lines(35), 
                    block
                }
            );
        }
    }

    fn send_whisper(game: &mut Game, sender_player: PlayerReference){
        let Some(StringSelection(text)) = ControllerID::whisper(sender_player).get_string_selection(game) else {return};
        let Some(PlayerListSelection(players)) = ControllerID::whisper_to_player(sender_player).get_player_list_selection(game) else {return};
        let Some(whispered_to_player) = players.first() else {
            sender_player.add_private_chat_message(game, ChatMessageVariant::InvalidWhisper);
            return
        };

        OnWhisper::new(sender_player, *whispered_to_player, text.clone()).invoke(game);
    }

    pub fn controller_parameters_map(game: &Game)->ControllerParametersMap{
        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|player|Self::one_player_controller_paraemeters_map(game, player))
        )
    }

    fn one_player_controller_paraemeters_map(game: &Game, player: PlayerReference)->ControllerParametersMap{
        let mut allowed_players: VecSet<PlayerReference> = PlayerReference::all_players(game)
            .filter(|p|
                if let RoleState::Cerenovous(cerenovous) = p.role_state(game){
                    cerenovous.currently_brained == Some(player)
                }else{false}
            )
            .collect();
        if allowed_players.is_empty() {
            allowed_players.insert(player);
        }

        
        //chat
        let chat = ControllerParametersMap::builder(game)
            .id(ControllerID::Chat{ player })
            .available_selection(AvailableStringSelection)
            .allow_players(allowed_players.clone())
            .build_map();

        let chat_is_block = ControllerParametersMap::builder(game)
            .id(ControllerID::ChatIsBlock { player })
            .available_selection(AvailableBooleanSelection)
            .allow_players(allowed_players.clone())
            .build_map();

        let send_chat = ControllerParametersMap::builder(game)
            .id(ControllerID::SendChat{ player })
            .available_selection(AvailableUnitSelection)
            .add_grayed_out_condition(player.get_current_send_chat_groups(game).is_empty())
            .allow_players(allowed_players.clone())
            .build_map();

        //whisper
        let whisper = ControllerParametersMap::builder(game)
            .id(ControllerID::Whisper{ player })
            .available_selection(AvailableStringSelection)
            .allow_players(allowed_players.clone())
            .build_map();

        let whisper_to_player = ControllerParametersMap::builder(game)
            .id(ControllerID::WhisperToPlayer { player })
            .single_player_selection_typical(player, false, true)
            .allow_players(allowed_players.clone())
            .build_map();

        let send_whisper = ControllerParametersMap::builder(game)
            .id(ControllerID::SendWhisper{ player })
            .available_selection(AvailableUnitSelection)
            .allow_players(allowed_players)
            .build_map();

        ControllerParametersMap::combine([chat, chat_is_block, send_chat, whisper, whisper_to_player, send_whisper])
    }
}