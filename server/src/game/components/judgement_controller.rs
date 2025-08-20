use crate::game::{
    controllers::{
        AvailableIntegerSelection, ControllerParametersMap, IntegerSelection,
    }, 
    modifiers::{ModifierType, Modifiers},
    player::PlayerReference, Game
};

pub struct JudgementController;

impl JudgementController{
    pub fn controller_parameters_map(game: &mut Game)->ControllerParametersMap{
        ControllerParametersMap::combine(
            PlayerReference::all_players(game)
                .map(|actor| Self::one_player_controller(game, actor))
        )
    }
    fn one_player_controller(game: &mut Game, actor: PlayerReference)->ControllerParametersMap{
        let abstain_enabled = Modifiers::is_enabled(game, ModifierType::Abstaining);

        ControllerParametersMap::builder(game)
            .id(crate::game::controllers::ControllerID::Judge { player: actor })
            .available_selection(AvailableIntegerSelection {
                min: 0,
                max: if abstain_enabled {2} else {1},
            })
            .add_grayed_out_condition(!actor.alive(game))
            .reset_on_phase_start(crate::game::phase::PhaseType::Judgement)
            .default_selection(IntegerSelection(if abstain_enabled {2} else {0}))
            .allow_players([actor])
            .build_map()
    }
}