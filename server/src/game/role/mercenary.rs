use serde::Serialize;

use crate::game::controllers::{AvailableIntegerSelection, IntegerSelection};
use crate::game::chat::ChatMessageVariant;
use crate::game::components::graves::grave::{Grave, GraveKiller};
use crate::game::event::on_midnight::{MidnightVariables, OnMidnightPriority};
use crate::game::attack_power::{AttackPower, DefensePower};
use rand::seq::SliceRandom;
use crate::game::player::PlayerReference;

use crate::game::visit::{Visit, VisitTag};
use crate::game::Game;
use crate::vec_set::VecSet;

use super::{ControllerID, ControllerParametersMap, Role, RoleStateImpl};


pub(super) const MAXIMUM_COUNT: Option<u8> = None;
pub(super) const DEFENSE: DefensePower = DefensePower::None;

#[derive(Clone, Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Mercenary{
    roles: VecSet<Role>,
    won: bool,
    attacks_remaining: u8,
}

impl RoleStateImpl for Mercenary {
    type ClientRoleState = Mercenary;
    fn new_state(game: &Game) -> Self {
        let mut available_roles = PlayerReference::all_players(game)
            .map(|p|p.role(game))
            .filter(|r|*r != Role::Mercenary)
            .collect::<VecSet<Role>>()
            .into_iter()
            .collect::<Vec<Role>>();

        available_roles.shuffle(&mut rand::rng());
        
        let attacks_remaining = crate::game::role::common_role::standard_charges(game);
        let mut roles = VecSet::new();
        for _ in 0..attacks_remaining {
            if let Some(role) = available_roles.pop(){
                roles.insert(role);
            }else{
                break;
            }
        }
        
        Self { won: false, roles, attacks_remaining }
    }
    fn on_midnight(self, game: &mut Game, midnight_variables: &mut MidnightVariables, actor_ref: PlayerReference, priority: OnMidnightPriority) {
        
        let visits = actor_ref.untagged_night_visits_cloned(midnight_variables);
        let Some(visit) = visits.first() else {return};

        match (priority, visit.tag) {
            (OnMidnightPriority::Kill, VisitTag::Role { role: Role::Mercenary, id: 2 }) => {
                if game.day_number() == 1 || self.attacks_remaining == 0 {return}
                visit.target.try_night_kill_single_attacker(
                    actor_ref,
                    game,
                    midnight_variables,
                    GraveKiller::Role(Role::Mercenary),
                    AttackPower::Basic,
                    true
                );
                actor_ref.set_role_state(game, Self{attacks_remaining: self.attacks_remaining.saturating_sub(1), ..self});
            },
            (OnMidnightPriority::Investigative, VisitTag::Role { role: Role::Mercenary, id: 1 }) => {
                actor_ref.push_night_message(
                    midnight_variables,
                    ChatMessageVariant::MercenaryResult{hit: self.roles.contains(&visit.target.role(game))}
                );
            },
            _ => {}
        }
    }
    fn controller_parameters_map(self, game: &Game, actor_ref: PlayerReference) -> ControllerParametersMap {
        let mut ctrl = ControllerParametersMap::builder(game)
            .id(ControllerID::role(actor_ref, Role::Mercenary, 0))
            .available_selection(AvailableIntegerSelection {
                min: 1,
                max: if game.day_number() <= 1 || self.attacks_remaining == 0 {1} else {2}
            })
            .default_selection(Self::controller_selection(actor_ref, game))
            .allow_players([actor_ref])
            .night_typical(actor_ref)
            .build_map();

        ctrl.combine_overwrite(
            ControllerParametersMap::builder(game)
                .id(ControllerID::role(actor_ref, Role::Mercenary, Self::controller_selection_controller_id(actor_ref, game)))
                .single_player_selection_typical(actor_ref, false, true)
                .night_typical(actor_ref)
                .build_map()
        );
        ctrl
    }
    fn convert_selection_to_visits(self, game: &Game, actor_ref: PlayerReference) -> Vec<Visit> {
        let controller_id = Self::controller_selection_controller_id(actor_ref, game);
        crate::game::role::common_role::convert_controller_selection_to_visits_visit_tag(
            game,
            actor_ref,
            ControllerID::role(actor_ref, Role::Mercenary, controller_id),
            controller_id == 2,
            VisitTag::Role{role: Role::Mercenary, id: controller_id}
        )
    }
    fn on_role_creation(self, game: &mut Game, actor_ref: PlayerReference) {
        for player in PlayerReference::all_players(game){
            if !self.roles.contains(&player.role(game)) {continue};
            player.add_private_chat_message(game, ChatMessageVariant::MercenaryYouAreAHit);
        }
        actor_ref.add_private_chat_message(
            game,
            ChatMessageVariant::MercenaryHits{roles: self.roles}
        );
    }
    fn on_any_death(self, game: &mut Game, actor_ref: PlayerReference, _dead_player_ref: PlayerReference) {
        self.check_win(game, actor_ref);
    }
    fn on_role_switch(self, game: &mut Game, actor_ref: PlayerReference, _player: PlayerReference, _new: super::RoleState, _old: super::RoleState) {   
        self.check_win(game, actor_ref);
    }
}
impl Mercenary{
    pub fn controller_selection(actor_ref: PlayerReference, game: &Game)->IntegerSelection{
        ControllerID::role(actor_ref, Role::Mercenary, 0)
            .get_integer_selection(game)
            .unwrap_or(&IntegerSelection(1))
            .clone()
    }
    pub fn controller_selection_controller_id(actor_ref: PlayerReference, game: &Game)->u8{
        Self::controller_selection(actor_ref, game)
            .0
            .try_into()
            .expect("1 and 2 should be only possible values here")
    }
    pub fn hits_dead(&self, game: &Game)->bool{
        self.roles
            .iter()
            .all(|role|
                PlayerReference::all_players(game)
                    .any(|p|p.role(game) == *role && !p.alive(game)) ||
                !PlayerReference::all_players(game).any(|p|p.role(game) == *role)
            )
    }
    pub fn check_win(&self, game: &mut Game, actor: PlayerReference){
        if self.hits_dead(game) && actor.alive(game) && !self.won {
            actor.set_role_state(game, Self{won: true, ..self.clone()});
            actor.die_and_add_grave(game, Grave::from_player_leave_town(game, actor));
        }
    }
    pub fn won(&self)->bool{
        self.won
    }
}