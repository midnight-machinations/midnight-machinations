use crate::{
    game::{
        controllers::{
            AvailablePlayerListSelection, AvailableRoleListSelection,
            AvailableSelectionKind, ControllerID, ControllerParameters
        }, components::{detained::Detained, insider_group::InsiderGroupID}, phase::PhaseType, player::PlayerReference, role::Role, Game
    },
    vec_set::VecSet
};

use super::ControllerParametersMap;

pub struct NoControllerSelection;

pub trait BuilderAvailableControllerSelectionState {
    type Selection;
}
impl<A: AvailableSelectionKind> BuilderAvailableControllerSelectionState for A {
    type Selection = A::Selection;
}
impl BuilderAvailableControllerSelectionState for NoControllerSelection {
    type Selection = NoControllerSelection;
}
pub trait BuilderIDState {}
impl BuilderIDState for () {}
impl BuilderIDState for ControllerID {}

pub struct ControllerParametersBuilder<'a, A: BuilderAvailableControllerSelectionState = NoControllerSelection, I: BuilderIDState = ()> {
    game: &'a Game,
    available: A,
    grayed_out: bool,
    reset_on_phase_start: Option<PhaseType>,
    dont_save: bool,
    default_selection: A::Selection,
    allowed_players: VecSet<PlayerReference>,
    id: I
}

impl<'a> ControllerParametersBuilder<'a, NoControllerSelection, ()> {
    pub fn new(game: &'a Game) -> Self {
        ControllerParametersBuilder {
            game,
            available: NoControllerSelection,
            grayed_out: false,
            reset_on_phase_start: None,
            dont_save: false,
            default_selection: NoControllerSelection,
            allowed_players: VecSet::new(),
            id: ()
        }
    }
}

impl<'a, I: BuilderIDState> ControllerParametersBuilder<'a, NoControllerSelection, I> {
    pub fn available_selection<A: AvailableSelectionKind>(self, available: A) -> ControllerParametersBuilder<'a, A, I> {
        let game = self.game;
        let default_selection = available.default_selection(game);

        ControllerParametersBuilder {
            game: self.game,
            available,
            grayed_out: self.grayed_out,
            reset_on_phase_start: self.reset_on_phase_start,
            dont_save: self.dont_save,
            default_selection,
            allowed_players: self.allowed_players,
            id: self.id
        }
    }

    pub fn single_player_selection_typical(
        self,
        actor_ref: PlayerReference,
        can_select_self: bool,
        can_select_insiders: bool,
    ) -> ControllerParametersBuilder<'a, AvailablePlayerListSelection, I> {
        self.player_list_selection_typical(actor_ref, can_select_self, can_select_insiders, false, false, Some(1))
    }
    pub fn player_list_selection_typical(
        self,
        actor_ref: PlayerReference,
        can_select_self: bool,
        can_select_insiders: bool,
        can_select_duplicates: bool,
        can_select_dead: bool,
        max_players: Option<u8>
    ) -> ControllerParametersBuilder<'a, AvailablePlayerListSelection, I> {
        let game = self.game;
        self.available_selection(AvailablePlayerListSelection {
            available_players: PlayerReference::all_players(game)
                .filter(|player|
                    if !player.alive(game){
                        can_select_dead
                    }else if *player == actor_ref{
                        can_select_self
                    }else if InsiderGroupID::in_same_group(game, actor_ref, *player){
                        can_select_insiders
                    }else{
                        true
                    }

                )
                .collect(),
            can_choose_duplicates: can_select_duplicates,
            max_players
        })
    }

    pub fn single_role_selection_typical(
        self, game: &Game, filter: impl FnMut(&Role) -> bool
    ) -> ControllerParametersBuilder<'a, AvailableRoleListSelection, I> {
        self.available_selection(AvailableRoleListSelection{
            available_roles: game.settings.enabled_roles.clone().into_iter().filter(filter).collect(),
            can_choose_duplicates: false,
            max_roles: Some(1)
        })
    }
    
}

impl<A: BuilderAvailableControllerSelectionState, I: BuilderIDState> ControllerParametersBuilder<'_, A, I> {
    pub fn night_typical(self, actor_ref: PlayerReference) -> Self {
        let game = self.game;
        self
            .add_grayed_out_condition(actor_ref.ability_deactivated_from_death(game) || Detained::is_detained(game, actor_ref))
            .reset_on_phase_start(PhaseType::Obituary)
            .allow_players([actor_ref])
    }

    pub fn reset_on_phase_start(self, phase: PhaseType) -> Self {
        Self {
            reset_on_phase_start: Some(phase),
            ..self
        }
    }

    pub fn add_grayed_out_condition(self, grayed_out: bool) -> Self {
        Self {
            grayed_out: self.grayed_out || grayed_out,
            ..self
        }
    }

    pub fn dont_save(self) -> Self {
        Self {
            dont_save: true,
            ..self
        }
    }

    pub fn allow_players(self, players: impl IntoIterator<Item=PlayerReference>) -> Self {
        Self {
            allowed_players: self.allowed_players.into_iter().chain(players).collect(),
            ..self
        }
    }
}

impl<A: AvailableSelectionKind, I: BuilderIDState> ControllerParametersBuilder<'_, A, I> {
    pub fn default_selection(self, default_selection: A::Selection) -> Self {
        Self {
            default_selection,
            ..self
        }
    }

    pub fn try_build(self) -> Option<ControllerParameters> {
        ControllerParameters::new(
            self.game, 
            self.available.into(), 
            self.grayed_out, 
            self.reset_on_phase_start, 
            self.dont_save, 
            self.default_selection.into(), 
            self.allowed_players
        )
    }
}

impl<'a, A: BuilderAvailableControllerSelectionState> ControllerParametersBuilder<'a, A, ()> {
    pub fn id(self, id: ControllerID) -> ControllerParametersBuilder<'a, A, ControllerID> {
        ControllerParametersBuilder {
            game: self.game,
            available: self.available,
            grayed_out: self.grayed_out,
            reset_on_phase_start: self.reset_on_phase_start,
            dont_save: self.dont_save,
            default_selection: self.default_selection,
            allowed_players: self.allowed_players,
            id
        }
    }
}

impl<A: AvailableSelectionKind> ControllerParametersBuilder<'_, A, ControllerID> {
    pub fn build_map(self) -> ControllerParametersMap {
        if let Some(single) = ControllerParameters::new(
            self.game,
            self.available.into(),
            self.grayed_out,
            self.reset_on_phase_start,
            self.dont_save,
            self.default_selection.into(),
            self.allowed_players
        ){
            ControllerParametersMap::new_controller(self.id, single)
        }else{
            ControllerParametersMap::default()
        }
    }
}