#[allow(dead_code)]
mod kit;

use mafia_server::game::{
    chat::{ChatComponent, ChatMessage, ChatMessageVariant},
    controllers::{ControllerID, ControllerInput, PlayerListSelection},
    phase::PhaseStateMachine,
    role::Role,
};

#[test]
fn night_feedback_is_marked_in_history_and_survives_requeue() {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Detective, Role::Mafioso, Role::Detective]);
    let detective = scenario.players[0];
    let mafia = scenario.players[1];
    let game = &mut scenario.game;
    PhaseStateMachine::next_phase(game, None);
    PhaseStateMachine::next_phase(game, None);
    ControllerInput::new(ControllerID::role(detective, Role::Detective, 0), PlayerListSelection(vec![mafia])).on_client_message(game, detective);
    PhaseStateMachine::next_phase(game, None);

    let result = detective.chat_messages(game).iter()
        .find(|message| matches!(message.variant, ChatMessageVariant::DetectiveResult { .. }))
        .expect("night result in chat history");
    assert!(result.night_result);
    assert_eq!(serde_json::to_value(result).unwrap()["nightResult"], true);
    assert!(detective.chat_messages(game).iter()
        .filter(|message| matches!(message.variant, ChatMessageVariant::PhaseChange { .. }))
        .all(|message| !message.night_result));

    while ChatComponent::not_sent_messages_pop_front(game, detective).is_some() {}
    ChatComponent::requeue_chat_messages(game, detective);
    assert!(ChatComponent::not_sent_messages(game, detective).iter()
        .any(|(_, message)| message.night_result && matches!(message.variant, ChatMessageVariant::DetectiveResult { .. })));
}

#[test]
fn night_result_marker_tracks_delivery_not_message_styling() {
    let ordinary = ChatMessage::new_private(ChatMessageVariant::Silenced);
    let overnight = ChatMessage::new_night_result(ChatMessageVariant::Silenced);
    assert!(!ordinary.night_result);
    assert!(overnight.night_result);
    assert!(serde_json::to_value(ordinary).unwrap().get("nightResult").is_none());
    assert_eq!(serde_json::to_value(overnight).unwrap()["nightResult"], true);
}
