import { ReactElement } from "react"
import React from "react"
import { Role, RoleState } from "../../../../../game/roleState.d"
import TwoRoleOutlineOptionSelectionMenu from "../AbilitySelectionTypes/TwoRoleOutlineOptionSelectionMenu"
import GAME_MANAGER from "../../../../.."
import { controllerIdToLinkWithPlayer, TwoRoleOutlineOptionSelection } from "../../../../../game/abilityInput"
import { usePlayerState } from "../../../../../components/useHooks"
import ListMap from "../../../../../ListMap"

export type AuditorResult = Role[];

export default function AuditorMenu(props: Readonly<{
    roleState: RoleState & {type: "auditor"}
}>): ReactElement {
    
    const myPlayerIndex = usePlayerState(
        state=>state.myIndex,
        ["yourPlayerIndex"]
    )!;

    const onInput = (selection: TwoRoleOutlineOptionSelection) => {
        GAME_MANAGER.sendAbilityInput({
            id: {
                type: "role",
                role: "auditor",
                player: myPlayerIndex,
                id: 0
            },
            selection: {
                type: "twoRoleOutlineOption",
                selection: selection
            }
        });
    }
    

    const savedAbilities = usePlayerState(
        playerState => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"]
    )!;
    
    const savedAbilitiesMap = new ListMap(savedAbilities, (k1, k2) => controllerIdToLinkWithPlayer(k1) === controllerIdToLinkWithPlayer(k2));

    let singleAbilitySave = savedAbilitiesMap.get({
        type: "role",
        role: "auditor",
        player: myPlayerIndex,
        id: 0
    });

    let newSelection;
    let newAvailable;
    if(
        singleAbilitySave !== null &&
        singleAbilitySave.selection.type === "twoRoleOutlineOption" &&
        singleAbilitySave.availableAbilityData.available.type === "twoRoleOutlineOption"
    ){
        newSelection = singleAbilitySave.selection.selection;
        newAvailable = singleAbilitySave.availableAbilityData.available.selection;
    } else {
        newSelection = undefined;
        newAvailable = undefined;
    }

    return <TwoRoleOutlineOptionSelectionMenu
        previouslyGivenResults={props.roleState.previouslyGivenResults}
        selection={newSelection}
        available={newAvailable}
        onChoose={onInput}
    />
}