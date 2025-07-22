import { ReactElement, useContext } from "react"
import React from "react"
import TwoRoleOutlineOptionSelectionMenu from "../AbilitySelectionTypes/TwoRoleOutlineOptionSelectionMenu"
import { controllerIdToLink, TwoRoleOutlineOptionSelection } from "../../../../../game/abilityInput"
import ListMap from "../../../../../ListMap"
import { WebsocketContext } from "../../../../WebsocketContext"
import { usePlayerState } from "../../../../../stateContext/useHooks"
import { Role, RoleState } from "../../../../../stateContext/stateType/roleState"

export type AuditorResult = Role[];

export default function AuditorMenu(props: Readonly<{
    roleState: RoleState & {type: "auditor"}
}>): ReactElement {
    const myPlayerIndex = usePlayerState()!.myIndex;
    const savedAbilities = usePlayerState()!.savedControllers;

    const savedAbilitiesMap = new ListMap(savedAbilities, (k1, k2) => controllerIdToLink(k1) === controllerIdToLink(k2));

    const { sendAbilityInput } = useContext(WebsocketContext)!;

    const onInput = (selection: TwoRoleOutlineOptionSelection) => {
        sendAbilityInput({
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