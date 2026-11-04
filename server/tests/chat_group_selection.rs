#[allow(dead_code)]
mod kit;

use mafia_server::{
    game::{
        chat::{ChatComponent, ChatGroup, ChatMessageVariant},
        components::insider_group::InsiderGroupID,
        controllers::{ChatGroupSelection, ControllerID, ControllerInput, ControllerSelection, StringSelection, UnitSelection},
        phase::{PhaseStateMachine, PhaseType},
        player::PlayerReference,
        role::Role,
        Game,
    },
    vec_set::VecSet,
};

fn night_game() -> kit::TestScenario {
    let mut scenario = kit::_init::create_basic_scenario(vec![Role::Medium, Role::Mafioso, Role::Detective, Role::Detective]);
    let medium = scenario.players[0];
    InsiderGroupID::Mafia.players_mut(&mut scenario.game).insert(medium);
    PhaseStateMachine::next_phase(&mut scenario.game, None);
    PhaseStateMachine::next_phase(&mut scenario.game, None);
    assert_eq!(scenario.game.current_phase().phase(), PhaseType::Night);
    scenario
}

fn choose(game: &mut Game, actor: PlayerReference, player: PlayerReference, group: ChatGroup) {
    ControllerInput::new(ControllerID::ChatGroup { player }, ChatGroupSelection(Some(group))).on_client_message(game, actor);
}

fn send(game: &mut Game, player: PlayerReference, text: &str) {
    ControllerInput::new(ControllerID::Chat { player }, StringSelection(text.to_owned())).on_client_message(game, player);
    ControllerInput::new(ControllerID::SendChat { player }, UnitSelection).on_client_message(game, player);
}

fn matching_groups(game: &Game, player: PlayerReference, text: &str) -> Vec<ChatGroup> {
    ChatComponent::chat_messages(game, player).iter().filter_map(|message| {
        match &message.variant {
            ChatMessageVariant::Normal { text: actual, .. } if actual == text => message.chat_group,
            _ => None,
        }
    }).collect()
}

#[test]
fn text_goes_to_one_selected_group_and_is_not_duplicated() {
    let mut scenario = night_game();
    let [medium, mafia, dead, _] = scenario.players.as_slice() else { unreachable!() };
    let game = &mut scenario.game;
    dead.die(game);
    let groups = medium.get_current_send_chat_groups(game);
    assert!(groups.contains(&ChatGroup::Mafia));
    assert!(groups.contains(&ChatGroup::Dead));

    choose(game, *medium, *medium, ChatGroup::Dead);
    send(game, *medium, "dead only");
    assert_eq!(matching_groups(game, *dead, "dead only"), vec![ChatGroup::Dead]);
    assert!(matching_groups(game, *mafia, "dead only").is_empty());
    assert_eq!(matching_groups(game, *medium, "dead only"), vec![ChatGroup::Dead]);

    choose(game, *medium, *medium, ChatGroup::Mafia);
    send(game, *medium, "mafia only");
    assert_eq!(matching_groups(game, *mafia, "mafia only"), vec![ChatGroup::Mafia]);
    assert!(matching_groups(game, *dead, "mafia only").is_empty());
    assert_eq!(matching_groups(game, *medium, "mafia only"), vec![ChatGroup::Mafia]);
}

#[test]
fn selection_is_retained_while_available_then_falls_back_to_all() {
    let mut scenario = night_game();
    let medium = scenario.players[0];
    let game = &mut scenario.game;
    choose(game, medium, medium, ChatGroup::Mafia);
    PhaseStateMachine::next_phase(game, None);
    assert_eq!(medium.selected_send_chat_group(game), Some(ChatGroup::Mafia));
    PhaseStateMachine::next_phase(game, None);
    assert_eq!(game.current_phase().phase(), PhaseType::Discussion);
    assert_eq!(medium.selected_send_chat_group(game), Some(ChatGroup::All));
    assert_eq!(ControllerID::ChatGroup { player: medium }.get_selection(game), Some(&ControllerSelection::ChatGroup(ChatGroupSelection(Some(ChatGroup::All)))));
}

#[test]
fn invalid_groups_and_other_players_cannot_change_the_selection() {
    let mut scenario = night_game();
    let medium = scenario.players[0];
    let stranger = scenario.players[3];
    let game = &mut scenario.game;
    choose(game, medium, medium, ChatGroup::Mafia);
    choose(game, medium, medium, ChatGroup::Jail);
    assert_eq!(medium.selected_send_chat_group(game), Some(ChatGroup::Mafia));
    choose(game, stranger, medium, ChatGroup::Dead);
    assert_eq!(medium.selected_send_chat_group(game), Some(ChatGroup::Mafia));
}

#[test]
fn permission_changes_update_the_destination_even_before_controller_refresh() {
    let mut scenario = night_game();
    let medium = scenario.players[0];
    let game = &mut scenario.game;
    choose(game, medium, medium, ChatGroup::Mafia);
    InsiderGroupID::Mafia.players_mut(game).remove(&medium);
    assert_eq!(medium.selected_send_chat_group(game), Some(ChatGroup::Dead));
}

#[test]
fn no_allowed_group_means_no_destination_or_text() {
    let mut scenario = night_game();
    let player = scenario.players[3];
    assert_eq!(player.selected_send_chat_group(&scenario.game), None);
    send(&mut scenario.game, player, "cannot send");
    for recipient in &scenario.players {
        assert!(matching_groups(&scenario.game, *recipient, "cannot send").is_empty());
    }
}

#[test]
fn fallback_prefers_all_then_first_and_preserves_valid_selections() {
    let groups = VecSet::from_iter([ChatGroup::Mafia, ChatGroup::All, ChatGroup::Dead]);
    assert_eq!(ChatGroup::select_available(&groups, None), Some(ChatGroup::All));
    assert_eq!(ChatGroup::select_available(&groups, Some(ChatGroup::Mafia)), Some(ChatGroup::Mafia));
    assert_eq!(ChatGroup::select_available(&groups, Some(ChatGroup::Jail)), Some(ChatGroup::All));
    assert_eq!(ChatGroup::select_available(&VecSet::from_iter([ChatGroup::Dead, ChatGroup::Mafia]), None), Some(ChatGroup::Dead));
    assert_eq!(ChatGroup::select_available(&VecSet::new(), None), None);
}
