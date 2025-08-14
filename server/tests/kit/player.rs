use mafia_server::{game::{ability_input::*, chat::ChatMessageVariant, game_conclusion::GameConclusion, phase::PhaseState, player::{PlayerIndex, PlayerReference}, role::{Role, RoleState}, verdict::Verdict, Game}, packet::ToServerPacket};

#[derive(Clone, Copy, Debug)]
pub struct TestPlayer(PlayerReference, *mut Game);

/// A macro to get the game from this TestPlayer.
/// ## Example:
/// ```
/// // In TestPlayer::can_day_target
/// assert!(self.0.can_day_target(game!(self), target.0));
/// 
/// game!(self).on_client_message(self.0.index(), 
///     ToServerPacket::DayTarget { player_index: target.index() }
/// );
/// ```
macro_rules! game {
    ($self:ident) => {unsafe {&mut *$self.1}}
}

impl TestPlayer {
    pub fn new(player: PlayerReference, game: &Game) -> Self {
        TestPlayer(player, game as *const Game as *mut Game)
    }

    pub fn index(&self) -> PlayerIndex {
        self.0.index()
    }

    pub fn player_ref(&self) -> PlayerReference {
        self.0
    }

    pub fn send_ability_input(&self, ability_input: AbilityInput) {
        game!(self).on_player_message(
            0, // This is only used for host stuff.
            self.0, 
            ToServerPacket::AbilityInput { ability_input }
        );
    }

    pub fn send_ability_input_integer_typical(&self, int: i8) {
        self.send_ability_input(AbilityInput::new(
            ControllerID::role(self.player_ref(), self.role(), 0),
            IntegerSelection(int),
        ))
    }

    pub fn send_ability_input_role_typical(&self, role: Vec<Role>){
        self.send_ability_input(AbilityInput::new(
            ControllerID::role(self.player_ref(), self.role(), 0),
            RoleListSelection(role),
        ))
    }

    pub fn send_ability_input_unit_typical(&self)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                UnitSelection
            )
        );
        true
    }

    pub fn send_ability_input_two_player_typical(&self, a: TestPlayer, b: TestPlayer)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                TwoPlayerOptionSelection(Some((a.player_ref(), b.player_ref())))
            )
        );
        true
    }

    pub fn send_ability_input_player_list_typical(&self, selection: impl Into<Vec<TestPlayer>>)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                PlayerListSelection(selection.into().iter().map(TestPlayer::player_ref).collect())
            )
        );
        true
    }

    pub fn send_ability_input_boolean_typical(&self, selection: bool)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), 0),
                BooleanSelection(selection)
            )
        );
        true
    }

    pub fn send_ability_input_player_list(&self, selection: impl Into<Vec<TestPlayer>>, id: RoleControllerID)->bool{
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::role(self.player_ref(), self.role(), id),
                PlayerListSelection(selection.into().iter().map(|p| p.player_ref()).collect())
            )
        );
        true
    }

    pub fn vote_for_player(&self, target: impl Into<Option<TestPlayer>>) {
        self.send_ability_input(
            AbilityInput::new(
                ControllerID::nominate(self.player_ref()),
                PlayerListSelection(target.into().iter().map(|p| p.player_ref()).collect())
            )
        );
    }
    pub fn set_verdict(&self, verdict: Verdict) {
        self.0.set_verdict(game!(self), verdict);
    }

    pub fn send_message(&self, message: &str) {
        game!(self).on_player_message(
            0, // This is only used for host stuff.
            self.0, 
            ToServerPacket::AbilityInput { ability_input: AbilityInput::new(
                ControllerID::Chat { player: self.0 },
                StringSelection(message.to_string())
            ) }
        );
        game!(self).on_player_message(
            0, // This is only used for host stuff.
            self.0, 
            ToServerPacket::AbilityInput { ability_input: AbilityInput::new(
                ControllerID::ChatIsBlock { player: self.0 },
                BooleanSelection(false)
            ) }
        );
        game!(self).on_player_message(
            0, // This is only used for host stuff.
            self.0, 
            ToServerPacket::AbilityInput { ability_input: AbilityInput::new(
                ControllerID::SendChat { player: self.0 },
                UnitSelection
            ) }
        );
    }

    pub fn alive(&self) -> bool {
        self.0.alive(game!(self))
    }

    pub fn received_blocked_message(&self) -> bool {
        let messages = self.get_messages_after_night(game!(self).day_number() - 1);

        messages.contains(&ChatMessageVariant::RoleBlocked) ||
        messages.contains(&ChatMessageVariant::Wardblocked)
    }

    pub fn get_messages(&self) -> Vec<ChatMessageVariant> {
        self.0.chat_messages(game!(self)).iter().map(|m|{
            m.variant().clone()
        }).collect()
    }

    pub fn get_messages_after_last_message(&self, last_message: ChatMessageVariant) -> Vec<ChatMessageVariant> {
        let mut found = false;
        let mut out = Vec::new();
        for message in self.get_messages().iter() {
            if *message == last_message {
                found = true;
            }else if found {
                out.push(message.clone());
            }
        }
        out
    }
    pub fn get_messages_after_night(&self, day_number: u8) -> Vec<ChatMessageVariant> {
        self.get_messages_after_last_message(
            ChatMessageVariant::PhaseChange { phase: PhaseState::Night, day_number }
        )
    }

    pub fn role(&self) -> Role {
        self.0.role(game!(self))
    }

    pub fn role_state(&self) -> &RoleState{
        self.0.role_state(game!(self))
    }

    pub fn set_role_state(&self, new_role_data: RoleState){
        self.0.set_role_state(game!(self), new_role_data);
    }

    pub fn get_won_game(&self) -> bool {
        if let Some(conclusion) = GameConclusion::game_is_over(game!(self)) {
            self.0.get_won_game(game!(self), conclusion)
        } else {
            panic!("Game is not over, cannot get won game status");
        }
    }
}

impl From<TestPlayer> for Vec<TestPlayer> {
    fn from(value: TestPlayer) -> Self {
        vec![value]
    }
}