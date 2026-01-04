use crate::game::abilities_component::ability::Ability;
use crate::game::abilities_component::ability_trait::AbilityTrait;
use crate::game::components::attack::night_attack::NightAttack;
use crate::vec_set;
use crate::game::prelude::*;

#[derive(Default, Clone, Debug)]
pub struct SyndicateGun {
    player_with_gun: Option<PlayerReference>
}

impl SyndicateGun {
    pub fn give_gun_to_player(game: &mut Game, player: PlayerReference) {
        AbilityID::SyndicateGun.set_ability(game, Some(Self{
            player_with_gun: Some(player)
        }));

        Tags::set_tagged(game, TagSetID::SyndicateGun, &vec_set![player]);
    }
    pub fn remove_gun(game: &mut Game) {
        AbilityID::SyndicateGun.set_ability(game, Some(Self{
            player_with_gun: None
        }));

        Tags::set_tagged(game, TagSetID::SyndicateGun, &vec_set![]);
    }

    pub fn player_with_gun(game: &Game) -> Option<PlayerReference> {
        if let Some(Ability::SyndicateGun(SyndicateGun { player_with_gun })) = AbilityID::SyndicateGun.get_ability(game) {
            *player_with_gun
        }else{
            None
        }
    }
    pub fn player_has_gun(game: &Game, player: PlayerReference) -> bool{
        Self::player_with_gun(game).is_some_and(|s|s==player)
    }

}
impl AbilityTrait for SyndicateGun {
    fn on_player_possessed(&self, game: &mut Game, _id: &AbilityID, event: &OnPlayerPossessed, fold: &mut OnMidnightFold, _priority: ()){
        if Some(event.possessed) != self.player_with_gun {
            return;
        }

        if Possession::possession_immune(&ControllerID::SyndicateGunShoot) { return; }
        Possession::possess_controller(game, ControllerID::SyndicateGunShoot, event.possessed, event.possessed_into);

        Visits::retain(fold, |v|v.tag != VisitTag::SyndicateGun || Some(v.visitor) != self.player_with_gun);

        let Some(player_with_gun) = self.player_with_gun else {return}; 

        let Some(PlayerListSelection(gun_target)) = ControllerID::syndicate_gun_item_shoot().get_player_list_selection(game) else {return};
        let Some(gun_target) = gun_target.first() else {return};

        Visits::add_visit(
            fold,
            Visit {
                visitor: player_with_gun,
                target: *gun_target,
                tag: VisitTag::SyndicateGun,
                attack: true,
                wardblock_immune: false,
                transport_immune: false,
                investigate_immune: false,
                indirect: false
            } 
        );
    }
    
    fn on_visit_wardblocked(&self, _game: &mut Game, _id: &AbilityID, event: &OnVisitWardblocked, midnight_variables: &mut OnMidnightFold, _priority: ()){
        Visits::retain(midnight_variables, |v|
            v.tag != VisitTag::SyndicateGun || v.visitor != event.visit.visitor
        );
    }
    fn on_player_roleblocked(&self, _game: &mut Game, _id: &AbilityID, event: &OnPlayerRoleblocked, midnight_variables: &mut OnMidnightFold, _priority: ()){
        Visits::retain(midnight_variables, |v|
            v.tag != VisitTag::SyndicateGun || v.visitor != event.player
        );
    }
    
    fn on_add_insider(&self, game: &mut Game, _id: &AbilityID, _event: &OnAddInsider, _fold: &mut (), _priority: ()){
        Tags::set_viewers(game, TagSetID::SyndicateGun, &InsiderGroupID::Mafia.players(game).clone());
    }
    fn on_remove_insider(&self, game: &mut Game, _id: &AbilityID, _event: &OnRemoveInsider, _fold: &mut (), _priority: ()){
        Tags::set_viewers(game, TagSetID::SyndicateGun, &InsiderGroupID::Mafia.players(game).clone());
    }
    fn on_any_death(&self, game: &mut Game, _id: &AbilityID, event: &OnAnyDeath, _fold: &mut (), _priority: ())  {
        if self.player_with_gun.is_some_and(|p|p==event.dead_player) {
            Self::remove_gun(game);

            let player = InsiderGroupID::Mafia.players(game).iter().find(|p|p.alive(game));
            if let Some(player) = player {
                SyndicateGun::give_gun_to_player(game, *player);
            }
        }
    }
    fn on_midnight(&self, game: &mut Game, _id: &AbilityID, _event: &OnMidnight, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority) {
        if game.day_number() <= 1 {return}
        match priority {
            OnMidnightPriority::TopPriority => {
                let Some(player_with_gun) = self.player_with_gun else {return}; 

                let Some(PlayerListSelection(gun_target)) = ControllerID::syndicate_gun_item_shoot().get_player_list_selection(game) else {return};
                let Some(gun_target) = gun_target.first() else {return};

                Visits::add_visit(
                    midnight_variables,
                    Visit {
                        visitor: player_with_gun,
                        target: *gun_target,
                        tag: VisitTag::SyndicateGun,
                        attack: true,
                        wardblock_immune: false,
                        transport_immune: false,
                        investigate_immune: false,
                        indirect: false
                    } 
                );
            }
            OnMidnightPriority::Kill => {
                for (attacker, target) in Visits::into_iter(midnight_variables)
                    .with_tag(VisitTag::SyndicateGun)
                    .map(|visit| (visit.visitor, visit.target))
                {
                    NightAttack::new()
                        .attackers([attacker])
                        .grave_killer(RoleSet::Mafia)
                        .attack(game, midnight_variables, target);
                }
            }
            _ => {}
        }
    }
    fn on_validated_ability_input_received(&self, game: &mut Game, _id: &AbilityID, event: &OnValidatedControllerInputReceived, _fold: &mut (), _priority: ()) {
        if let Some(player_with_gun) = self.player_with_gun {
            if event.actor_ref != player_with_gun {
                return;
            }
        }else{
            return;
        }

        let Some(PlayerListSelection(target)) = event.input
            .get_player_list_selection_if_id(ControllerID::SyndicateGunGive)
        else {return};
        let Some(target) = target.first() else {return};

        if
            event.actor_ref != *target &&
            target.alive(game) &&
            InsiderGroupID::Mafia.contains_player(game, *target) 
        {
            SyndicateGun::give_gun_to_player(game, *target);
        }
    }

    fn controller_parameters_map(&self, game: &Game, _id: &AbilityID) -> ControllerParametersMap {
        if let Some(player_with_gun) = self.player_with_gun {
            ControllerParametersMap::combine([
                ControllerParametersMap::builder(game)
                    .id(ControllerID::syndicate_gun_item_shoot())
                    .single_player_selection_typical(player_with_gun, false, false)
                    .night_typical(player_with_gun)
                    .add_grayed_out_condition(game.day_number() <= 1)
                    .build_map(),
                ControllerParametersMap::builder(game)
                    .id(ControllerID::syndicate_gun_item_give())
                    .available_selection(AvailablePlayerListSelection {
                        available_players: PlayerReference::all_players(game)
                            .filter(|target|
                                player_with_gun != *target &&
                                target.alive(game) &&
                                InsiderGroupID::Mafia.contains_player(game, *target))
                            .collect(),
                        can_choose_duplicates: false,
                        max_players: Some(1)
                    })
                    .add_grayed_out_condition(
                        Detained::is_detained(game, player_with_gun) ||
                        player_with_gun.ability_deactivated_from_death(game)
                    )
                    .reset_on_phase_start(PhaseType::Obituary)
                    .dont_save()
                    .allow_players([player_with_gun])
                    .build_map()
            ])
        }else{
            ControllerParametersMap::default()
        }
    }
}
impl From<SyndicateGun> for Ability {
    fn from(role_struct: SyndicateGun) -> Self {
        Ability::SyndicateGun(role_struct)
    }
}