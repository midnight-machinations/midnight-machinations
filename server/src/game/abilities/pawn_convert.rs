use crate::game::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct PawnConvert;

const fn pawn_convert_controller(player: PlayerReference) -> ControllerID {
    ControllerID::Role { player, role: Role::Pawn, id: 0 }
}

impl AbilityTrait for PawnConvert {
    fn on_midnight(
        &self,
        game: &mut Game,
        _id: &AbilityID,
        _event: &OnMidnight,
        midnight_variables: &mut OnMidnightFold,
        priority: OnMidnightPriority
    )
    {
        if priority != OnMidnightPriority::Convert {return}
        PlayerReference::all_players(game)
            .filter_map(|player|pawn_convert_controller(player).get_role_list_selection_first(game).map(|role|(player, role)))
            .for_each(|(player, role)|
                if (AbilityID::Role { role: Role::Pawn, player }).exists(game) {
                    player.set_night_convert_role_to(midnight_variables, Some(role.default_state()));
                }else{
                    polymorph(game, midnight_variables, player);
                }
            );
    }
    fn controller_parameters_map(&self, game: &Game, _id: &AbilityID) -> ControllerParametersMap {
        PlayerReference::all_players(game).fold(
            ControllerParametersMap::default(),
            |map, player|{
                map.combine_overwrite_owned(
                    ControllerParametersMap::builder(game)
                        .id(pawn_convert_controller(player))
                        .single_role_selection_typical(game, |role|
                            RoleSet::Minions.get_roles().contains(role) &&
                            game.settings.enabled_roles.contains(role) &&
                            *role != Role::Pawn
                        )
                        .night_typical(player)
                        .build_map()
                )
            }
        )
    }
}
impl From<PawnConvert> for Ability {
    fn from(role_struct: PawnConvert) -> Self {
        Ability::PawnConvert(role_struct)
    }
}


fn polymorph(
    game: &Game, 
    midnight_variables: &mut OnMidnightFold,
    player: PlayerReference
) {
    let role = player.role(game);
    if RoleSet::MafiaKilling.get_roles().contains(&role) {
        player.set_night_convert_role_to(midnight_variables, Some(Role::Mafioso.default_state()));
    }
    else if RoleSet::Mafia.get_roles().contains(&role) {
        player.set_night_convert_role_to(midnight_variables, Some(Role::Goon.default_state()));
    }
    else if RoleSet::Town.get_roles().contains(&role) {
        player.set_night_convert_role_to(midnight_variables, Some(Role::Villager.default_state()));
    }
    else if RoleSet::Fiends.get_roles().contains(&role) {
        player.set_night_convert_role_to(midnight_variables, Some(Role::SerialKiller.default_state()));
    }
    else if RoleSet::Minions.get_roles().contains(&role) {
        player.set_night_convert_role_to(midnight_variables, Some(Role::Lackey.default_state()));
    }
}