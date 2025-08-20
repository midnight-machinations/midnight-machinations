use crate::{
    game::{
        controllers::*, attack_power::AttackPower, components::graves::grave::GraveKiller, event::on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, game_conclusion::GameConclusion, phase::PhaseType, player::PlayerReference, role::Role, role_list::RoleSet, Game
    },
    vec_map::VecMap, vec_set::VecSet
};

#[derive(Clone)]
pub struct Pitchfork{
    pitchfork_owners: VecSet<PlayerReference>,

    pitchfork_uses_remaining: u8,

    angry_mobbed_player: Option<PlayerReference>,
}

impl Game{
    pub fn pitchfork(&self) -> &Pitchfork{
        &self.pitchfork
    }
    pub fn set_pitchfork(&mut self, pitchfork: Pitchfork){
        self.pitchfork = pitchfork;
    }
}

impl Default for Pitchfork{
    fn default() -> Self {
        Self {
            pitchfork_owners: Default::default(),
            pitchfork_uses_remaining: 3,
            angry_mobbed_player: Default::default()
        }
    }
}

impl Pitchfork{
    pub fn new(num_players: u8)->Self{
        Self{
            pitchfork_uses_remaining: num_players.div_ceil(5),
            ..Self::default()
        }
    }
    pub fn controller_parameters_map(game: &Game)->ControllerParametersMap{
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
                        !player.alive(game) ||
                        !player.win_condition(game).is_loyalist_for(GameConclusion::Town)
                    )
                    .reset_on_phase_start(PhaseType::Obituary)
                    .allow_players([player])
                    .build_map()
            )
        )
    }
    

    pub fn before_phase_end(game: &mut Game, phase: PhaseType){
        if phase == PhaseType::Night {
            Pitchfork::set_angry_mobbed_player(game, None);
            if Pitchfork::pitchfork_uses_remaining(game) > 0 {
                if let Some(target) = Pitchfork::player_is_voted(game){
                    Pitchfork::set_angry_mobbed_player(game, Some(target));
                }
            }
        }
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        if priority != OnMidnightPriority::Kill {return;}
        if game.day_number() <= 1 {return;}
        if Pitchfork::usable_pitchfork_owners(game, midnight_variables).is_empty() {return;}
        
        if let Some(target) = Pitchfork::angry_mobbed_player(game) {
            target.try_night_kill(
                &Pitchfork::usable_pitchfork_owners(game, midnight_variables), 
                game, midnight_variables,
                GraveKiller::RoleSet(RoleSet::Town), 
                AttackPower::ProtectionPiercing, 
                false
            );
            Pitchfork::set_pitchfork_uses_remaining(game,
                Pitchfork::pitchfork_uses_remaining(game).saturating_sub(1)
            );
        }
    }

    pub fn usable_pitchfork_owners(game: &Game, midnight_variables: &MidnightVariables) -> VecSet<PlayerReference> {
        Pitchfork::pitchfork_owners(game).iter()
            .filter(|p|p.alive(game) && !p.night_blocked(midnight_variables))
            .copied().collect()
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
            if count >= Pitchfork::number_of_votes_needed(game) {return Some(*target);}
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


    pub fn angry_mobbed_player(game: &Game) -> Option<PlayerReference>{
        game.pitchfork.angry_mobbed_player
    }
    pub fn set_angry_mobbed_player(game: &mut Game, player_ref: Option<PlayerReference>){
        let mut pitchfork = game.pitchfork().clone();
        pitchfork.angry_mobbed_player = player_ref;
        game.set_pitchfork(pitchfork);
    }
    pub fn pitchfork_owners(game: &Game) -> VecSet<PlayerReference>{
        game.pitchfork().pitchfork_owners.clone()
    }
    pub fn has_pitchfork(game: &Game, player_ref: PlayerReference) -> bool{
        game.pitchfork().pitchfork_owners.contains(&player_ref)
    }
    pub fn add_pitchfork(game: &mut Game, player_ref: PlayerReference){
        let mut pitchfork = game.pitchfork().clone();
        pitchfork.pitchfork_owners.insert(player_ref);
        game.set_pitchfork(pitchfork);
    }
    pub fn remove_pitchfork(game: &mut Game, player_ref: PlayerReference){
        let mut pitchfork = game.pitchfork().clone();
        pitchfork.pitchfork_owners.remove(&player_ref);
        game.set_pitchfork(pitchfork);
    }
    pub fn pitchfork_uses_remaining(game: &Game) -> u8{
        game.pitchfork().pitchfork_uses_remaining
    }
    pub fn set_pitchfork_uses_remaining(game: &mut Game, uses: u8){
        let mut pitchfork = game.pitchfork().clone();
        pitchfork.pitchfork_uses_remaining = uses;
        game.set_pitchfork(pitchfork);
    }
}