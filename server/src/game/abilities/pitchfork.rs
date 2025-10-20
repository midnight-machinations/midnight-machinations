use crate::{
    game::{
        abilities_component::{ability::Ability, ability_id::AbilityID, ability_trait::AbilityTrait},
        attack_power::AttackPower, components::{graves::grave::GraveKiller, pitchfork_item::PitchforkItemComponent},
        controllers::{AvailablePlayerListSelection, ControllerID, ControllerParametersMap, PlayerListSelection},
        event::{before_phase_end::BeforePhaseEnd, on_midnight::{OnMidnightFold, OnMidnight, OnMidnightPriority}},
        game_conclusion::GameConclusion, phase::PhaseType, player::PlayerReference, role::{common_role, Role}, role_list::RoleSet, Game
    },
    vec_map::VecMap
};

#[derive(Clone, Default, Debug)]
pub struct PitchforkAbility{
    charges: u8,
    angry_mobbed_player: Option<PlayerReference>,
}
impl From<PitchforkAbility> for Ability where PitchforkAbility: AbilityTrait {
    fn from(ability: PitchforkAbility) -> Self {
        Ability::Pitchfork(ability)
    }
}
impl PitchforkAbility{
    pub fn new_state(game: &mut Game)->Self{
        Self { charges: common_role::standard_charges(game), angry_mobbed_player: None }
    }

    
    pub fn player_is_voted(game: &Game) -> Option<PlayerReference> {
        let mut votes: VecMap<PlayerReference, u8> = VecMap::new();


        for voter in PlayerReference::all_players(game){
            let Some(PlayerListSelection(target)) = ControllerID::pitchfork_vote(voter)
                .get_player_list_selection(game)
                else {continue};
            let Some(target) = target.first() else {continue};
            if 
                !voter.alive(game) || 
                !voter.win_condition(game).is_loyalist_for(GameConclusion::Town) ||
                !target.alive(game)
            {continue;}


            let count: u8 = if let Some(count) = votes.get(target){
                count.saturating_add(1)
            }else{
                1
            };
            if count >= Self::number_of_votes_needed(game) {return Some(*target);}
            votes.insert(*target, count);
        }
        None
    }

    pub fn number_of_votes_needed(game: &Game) -> u8 {
        let eligible_voters = PlayerReference::all_players(game).filter(|p|
            p.alive(game) && p.win_condition(game).is_loyalist_for(GameConclusion::Town)
        ).count().try_into().unwrap_or(u8::MAX);
        // equivalent to x - (x - (x + 1)/3)/2 to prevent overflow issues
        let two_thirds = eligible_voters
        .saturating_sub(
            eligible_voters
            .saturating_sub(
                eligible_voters
                .saturating_add(1)
                .saturating_div(3)
            )
            .saturating_div(2)
        );
        if two_thirds == 0 {1} else {two_thirds}
    }
}
impl AbilityTrait for PitchforkAbility{
    fn on_midnight(&self, game: &mut Game, _id: &AbilityID, _event: &OnMidnight, midnight_variables: &mut OnMidnightFold, priority: OnMidnightPriority){
        if priority != OnMidnightPriority::Kill {return;}
        if game.day_number() <= 1 {return;}
        let pitchfork_players = PitchforkItemComponent::players_with_pitchfork(game, midnight_variables);
        if pitchfork_players.is_empty() {return;}
        
        let mut ability = self.clone();
        if let Some(target) = ability.angry_mobbed_player {
            target.try_night_kill(
                pitchfork_players.clone(), 
                game, midnight_variables,
                GraveKiller::RoleSet(RoleSet::Town), 
                AttackPower::ProtectionPiercing, 
                false
            );
            ability.charges = ability.charges.saturating_sub(1);
        }
        AbilityID::Pitchfork.set_ability(game, Some(ability));
    }
    fn before_phase_end(&self, game: &mut Game, _id: &AbilityID, event: &BeforePhaseEnd, _fold: &mut (), _priority: ()){
        if event.phase != PhaseType::Night {return};
        
        let mut ability = self.clone();
        ability.angry_mobbed_player = if ability.charges > 0 && let Some(target) = Self::player_is_voted(game){
            Some(target)
        }else{
            None
        };
        AbilityID::Pitchfork.set_ability(game, Some(ability));
    }

    fn controller_parameters_map(&self, game: &Game, _id: &AbilityID)->ControllerParametersMap{
        if !game.settings.enabled_roles.contains(&Role::Rabblerouser) {
            return ControllerParametersMap::default();
        }

        ControllerParametersMap::combine(
            PlayerReference::all_players(game).map(|player|
                ControllerParametersMap::builder(game)
                    .id(ControllerID::pitchfork_vote(player))
                    .available_selection(AvailablePlayerListSelection{
                        available_players: PlayerReference::all_players(game)
                            .filter(|p|p.alive(game))
                            .collect(),
                        can_choose_duplicates: false,
                        max_players: Some(1)
                    })
                    .add_grayed_out_condition(
                        game.day_number() == 1 ||
                        player.ability_deactivated_from_death(game) ||
                        !player.win_condition(game).is_loyalist_for(GameConclusion::Town)
                    )
                    .reset_on_phase_start(PhaseType::Obituary)
                    .allow_players([player])
                    .build_map()
            )
        )
    }
}