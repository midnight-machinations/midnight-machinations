import type { ReactElement } from "react";
import GAME_MANAGER from "../../../../..";
import { usePlayerState } from "../../../../../components/useHooks";
import { controllerIdToLinkWithPlayer, type TwoRoleOutlineOptionSelection } from "../../../../../game/controllerInput";
import type { Role, RoleState } from "../../../../../game/roleState.d";
import ListMap from "../../../../../ListMap";
import TwoRoleOutlineOptionSelectionMenu from "../ControllerSelectionTypes/TwoRoleOutlineOptionSelectionMenu";

export type AuditorResult = Role[];

export default function AuditorMenu(
    props: Readonly<{
        roleState: RoleState & { type: "auditor" };
    }>,
): ReactElement {
    const myPlayerIndex = usePlayerState((state) => state.myIndex, ["yourPlayerIndex"])!;

    const onInput = (selection: TwoRoleOutlineOptionSelection) => {
        GAME_MANAGER.sendControllerInput({
            id: {
                type: "role",
                role: "auditor",
                player: myPlayerIndex,
                id: 0,
            },
            selection: {
                type: "twoRoleOutlineOption",
                selection: selection,
            },
        });
    };

    const savedAbilities = usePlayerState(
        (playerState) => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"],
    )!;

    const savedAbilitiesMap = new ListMap(
        savedAbilities,
        (k1, k2) => controllerIdToLinkWithPlayer(k1) === controllerIdToLinkWithPlayer(k2),
    );

    const singleAbilitySave = savedAbilitiesMap.get({
        type: "role",
        role: "auditor",
        player: myPlayerIndex,
        id: 0,
    });

    let newSelection;
    let newAvailable;
    if (
        singleAbilitySave !== null &&
        singleAbilitySave.selection.type === "twoRoleOutlineOption" &&
        singleAbilitySave.parameters.available.type === "twoRoleOutlineOption"
    ) {
        newSelection = singleAbilitySave.selection.selection;
        newAvailable = singleAbilitySave.parameters.available.selection;
    } else {
        newSelection = undefined;
        newAvailable = undefined;
    }

    return (
        <TwoRoleOutlineOptionSelectionMenu
            previouslyGivenResults={props.roleState.previouslyGivenResults}
            selection={newSelection}
            available={newAvailable}
            onChoose={onInput}
        />
    );
}
