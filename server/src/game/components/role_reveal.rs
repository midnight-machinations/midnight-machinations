use crate::{
    game::{
        chat::ChatMessageVariant, components::player_component::PlayerComponent, event::{on_conceal_role::OnConcealRole, on_role_switch::OnRoleSwitch, AsInvokable as _, Invokable as _},
        player::PlayerReference, role::Role, Game
    }, packet::ToClientPacket, vec_map::VecMap, vec_set::VecSet
};

pub struct RevealedPlayers(VecSet<PlayerReference>);
pub type RevealedPlayersComponent = PlayerComponent<RevealedPlayers>;

impl RevealedPlayersComponent {
    /// # Safety
    /// num_players must be correct
    pub unsafe fn new(num_players: u8) -> Self {
        unsafe {
            PlayerComponent::<RevealedPlayers>::new_component_box(
                num_players,
                |_| RevealedPlayers(VecSet::new())
            )
        }
    }
    pub fn on_role_switch(game: &mut Game, event: &OnRoleSwitch, _fold: &mut (), _priority: ()){
        if event.old.role() == event.new.role() {return;}

        for player_ref in PlayerReference::all_players(game){
            player_ref.conceal_players_role(game, event.player);
        }
    }
}
impl PlayerReference{
    pub fn revealed_players<'a>(&self, game: &'a Game) -> &'a VecSet<PlayerReference>{
        &game.revealed_players.get(*self).0
    }
    pub fn revealed_players_map(&self, game: &Game) -> VecMap<PlayerReference, Role> {
        let mut map = VecMap::new();
        for player in self.revealed_players(game).iter() {
            map.insert(*player, player.role(game));
        }
        map
    }
    fn revealed_players_mut<'a>(&self, game: &'a mut Game) -> &'a mut VecSet<PlayerReference>{
        &mut game.revealed_players.get_mut(*self).0
    }
    pub fn reveal_players_role(&self, game: &mut Game, revealed_player: PlayerReference){
        if
            revealed_player != *self &&
            revealed_player.alive(game) &&
            self.revealed_players_mut(game).insert(revealed_player).is_none()
        {
            self.add_private_chat_message(game, ChatMessageVariant::PlayersRoleRevealed { player: revealed_player, role: revealed_player.role(game) })
        }

        self.send_packet(game, ToClientPacket::YourRoleLabels{
            role_labels: self.revealed_players_map(game)
        });
    }
    pub fn conceal_players_role(&self, game: &mut Game, concealed_player: PlayerReference){
        if self.revealed_players_mut(game).remove(&concealed_player).is_some() {
            self.add_private_chat_message(game, ChatMessageVariant::PlayersRoleConcealed { player: concealed_player })
        }

        self.send_packet(game, ToClientPacket::YourRoleLabels{
            role_labels: self.revealed_players_map(game)
        });

        OnConcealRole::new(*self, concealed_player).as_invokable().invoke(game);
    }
}