use std::collections::HashSet;

use crate::{game::{
    attack_power::AttackPower, chat::ChatMessageVariant,
    event::{on_add_insider::OnAddInsider, on_midnight::{MidnightVariables, OnMidnight, OnMidnightPriority}, on_remove_insider::OnRemoveInsider},
    game_conclusion::GameConclusion, player::PlayerReference, role::Role, Game
}, vec_set::VecSet};

use super::{insider_group::InsiderGroupID, tags::Tags, win_condition::WinCondition};

impl Game{
    pub fn puppeteer_marionette(&self)->&PuppeteerMarionette{
        &self.puppeteer_marionette
    }
    pub fn set_puppeteer_marionette(&mut self, puppeteer_marionette: PuppeteerMarionette){
        self.puppeteer_marionette = puppeteer_marionette;
    }
}

#[derive(Default, Clone)]
pub struct PuppeteerMarionette{
    marionettes: HashSet<PlayerReference>,
}
impl PuppeteerMarionette{
    pub fn string(game: &mut Game, midnight_variables: &mut MidnightVariables, player: PlayerReference)->bool{
        let mut puppeteer_marionette = game.puppeteer_marionette().clone();

        if player.role(game) == Role::Puppeteer {return false;}
        if !puppeteer_marionette.marionettes.insert(player){return false;}
        Tags::add_tag(game, super::tags::TagSetID::PuppeteerMarionette, player);

        game.set_puppeteer_marionette(puppeteer_marionette);
        InsiderGroupID::Puppeteer.add_player_to_revealed_group(game, player);
        player.set_win_condition(game, WinCondition::GameConclusionReached { win_if_any: vec![GameConclusion::Fiends].into_iter().collect() });

        for fiend in PuppeteerMarionette::marionettes_and_puppeteer(game){
            fiend.push_night_message(midnight_variables, ChatMessageVariant::PuppeteerPlayerIsNowMarionette{player: player.index()});
        }

        true
    }

    pub fn kill_marionettes(game: &mut Game, midnight_variables: &mut MidnightVariables){
        let marionettes = 
            game.puppeteer_marionette()
                .marionettes
                .iter()
                .filter(|p|p.alive(game))
                .copied()
                .collect::<Vec<_>>();

        PuppeteerMarionette::attack_players(game, midnight_variables, marionettes, AttackPower::ProtectionPiercing);
    }
    fn attack_players(game: &mut Game, midnight_variables: &mut MidnightVariables, players: Vec<PlayerReference>, attack_power: AttackPower){
        
        let puppeteers: VecSet<_> = PlayerReference::all_players(game)
            .filter(|p|p.role(game)==Role::Puppeteer)
            .collect();

        for player in players{
            player.try_night_kill(&puppeteers, game, midnight_variables, crate::game::grave::GraveKiller::Role(Role::Puppeteer), attack_power, true);
        }
    }

    pub fn is_marionette(game: &Game, player: PlayerReference)->bool{
        game.puppeteer_marionette().marionettes.contains(&player)
    }
    pub fn marionettes(game: &Game)->HashSet<PlayerReference>{
        PlayerReference::all_players(game)
            .filter(|p|
                game.puppeteer_marionette().marionettes.contains(p)
            )
            .collect()
    }
    pub fn puppeteers(game: &Game)->HashSet<PlayerReference>{
        PlayerReference::all_players(game)
            .filter(|p|p.role(game)==Role::Puppeteer)
            .collect()
    }
    pub fn marionettes_and_puppeteer(game: &Game)->HashSet<PlayerReference>{
        let mut marionettes_and_puppeteer = PuppeteerMarionette::marionettes(game);
        marionettes_and_puppeteer.extend(PuppeteerMarionette::puppeteers(game));
        marionettes_and_puppeteer
    }



    //event listeners
    pub fn on_add_insider(game: &mut Game, _event: &OnAddInsider, _fold: &mut (), _priority: ()){
        Tags::set_viewers(game, super::tags::TagSetID::PuppeteerMarionette, &InsiderGroupID::Puppeteer.players(game).clone());
    }
    pub fn on_remove_insider(game: &mut Game, _event: &OnRemoveInsider, _fold: &mut (), _priority: ()){
        Tags::set_viewers(game, super::tags::TagSetID::PuppeteerMarionette, &InsiderGroupID::Puppeteer.players(game).clone());
    }
    pub fn on_midnight(game: &mut Game, _event: &OnMidnight, midnight_variables: &mut MidnightVariables, priority: OnMidnightPriority){
        if priority == OnMidnightPriority::Kill{
            PuppeteerMarionette::kill_marionettes(game, midnight_variables);
        }
    }
}