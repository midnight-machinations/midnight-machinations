use serde::{Deserialize, Serialize};

use super::{
    selection_type::{
        kira_selection::KiraSelection, 
        two_player_option_selection::TwoPlayerOptionSelection,
        two_role_option_selection::TwoRoleOptionSelection,
        two_role_outline_option_selection::TwoRoleOutlineOptionSelection,
        BooleanSelection
    },
    *,
    ChatMessageSelection, IntegerSelection, PlayerListSelection, StringSelection, UnitSelection
};

macro_rules! selection_kinds {
    (
        $($name:ident: $available_kind:ident, $kind:ident);*
    ) => {
        #[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
        #[serde(rename_all = "camelCase")]
        #[serde(tag="type", content="selection")]
        pub enum AvailableControllerSelection {
            $($name($available_kind)),*
        }

        $(
            impl From<$available_kind> for AvailableControllerSelection {
                fn from(value: $available_kind) -> AvailableControllerSelection {
                    AvailableControllerSelection::$name(value)
                }
            }
        )*

        #[derive(Clone, Serialize, Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq)]
        #[serde(rename_all = "camelCase")]
        #[serde(tag="type", content="selection")]
        pub enum ControllerSelection{
            $($name($kind)),*
        }

        $(
            impl From<$kind> for ControllerSelection {
                fn from(value: $kind) -> ControllerSelection {
                    ControllerSelection::$name(value)
                }
            }
        )*
        

        impl AvailableControllerSelection{
            pub fn validate_selection(&self, game: &Game, selection: &ControllerSelection)->bool {
                match self {
                    $(Self::$name(available) => {
                        let ControllerSelection::$name(selection) = selection else {return false};
                        available.validate_selection(game, selection)
                    }),*
                }
            }
        }
    }
}

selection_kinds! {
    Unit: AvailableUnitSelection, UnitSelection;
    Boolean: AvailableBooleanSelection, BooleanSelection;

    PlayerList: AvailablePlayerListSelection, PlayerListSelection;
    TwoPlayerOption: AvailableTwoPlayerOptionSelection, TwoPlayerOptionSelection;

    RoleList: AvailableRoleListSelection, RoleListSelection;
    TwoRoleOption: AvailableTwoRoleOptionSelection, TwoRoleOptionSelection;

    TwoRoleOutlineOption: AvailableTwoRoleOutlineOptionSelection, TwoRoleOutlineOptionSelection;
    String: AvailableStringSelection, StringSelection;
    Integer: AvailableIntegerSelection, IntegerSelection;
    Kira: AvailableKiraSelection, KiraSelection;
    ChatMessage: AvailableChatMessageSelection, ChatMessageSelection
}
