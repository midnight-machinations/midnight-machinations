import { type ReactElement, useContext, useMemo } from "react";
import GAME_MANAGER from "../../../..";
import { Button } from "../../../../components/Button";
import StyledText from "../../../../components/StyledText";
import {
    type AvailableControllerSelection,
    type ControllerID,
    type ControllerSelection,
    controllerIdToLink,
    controllerIdToLinkWithPlayer,
    type GraveDeathCausesSelection,
    type IntegerSelection,
    type PlayerListSelection,
    type RoleListSelection,
    type SavedController,
    type StringSelection,
    singleAbilityJsonData,
    type TwoPlayerOptionSelection,
    type TwoRoleOptionSelection,
    type TwoRoleOutlineOptionSelection,
    translateControllerID,
    translateControllerIDNoRole,
} from "../../../../game/controllerInput";
import KiraSelectionMenu, { type KiraSelection } from "./ControllerSelectionTypes/KiraSelectionMenu";
import RoleListSelectionMenu from "./ControllerSelectionTypes/RoleListSelectionMenu";
import TwoPlayerOptionSelectionMenu from "./ControllerSelectionTypes/TwoPlayerOptionSelectionMenu";
import TwoRoleOptionSelectionMenu from "./ControllerSelectionTypes/TwoRoleOptionSelectionMenu";
import TwoRoleOutlineOptionSelectionMenu from "./ControllerSelectionTypes/TwoRoleOutlineOptionSelectionMenu";
import "./genericAbilityMenu.css";
import DetailsSummary from "../../../../components/DetailsSummary";
import type { PlayerIndex } from "../../../../game/gameState.d";
import translate from "../../../../game/lang";
import type { Role } from "../../../../game/roleState.d";
import ListMap from "../../../../ListMap";
import BooleanSelectionMenu from "./ControllerSelectionTypes/BooleanSelectionMenu";
import IntegerSelectionMenu from "./ControllerSelectionTypes/IntegerSelectionMenu";
import PlayerListSelectionMenu from "./ControllerSelectionTypes/PlayerListSelectionMenu";
import StringSelectionMenu from "./ControllerSelectionTypes/StringSelectionMenu";
import "./ControllerSelectionTypes/genericListController.css";
import ChatGroupSelect from "../../../../components/ChatGroupSelect";
import { useGameState, usePlayerState } from "../../../../components/useHooks";
import { setWikiSearchPage } from "../../../../components/Wiki";
import { loadSettingsParsed } from "../../../../game/localStorage";
import { AnchorControllerContext } from "../../../Anchor";
import { MenuControllerContext } from "../../GameScreen";
import GraveDeathCausesSelectionMenu from "./ControllerSelectionTypes/GraveDeathCausesSelectionMenu";

type GroupName = `${PlayerIndex}/${Role}` | "syndicate" | "chat" | "vote" | "whisper" | ControllerID["type"];

type ControllerGroupsMap = ListMap<GroupName, ListMap<ControllerID, SavedController>>;

export function getGroupNameFromControllerID(id: ControllerID): GroupName {
    switch (id.type) {
        case "role":
            return ("role/" + id.player + "/" + id.role) as `${PlayerIndex}/${Role}`;
        case "syndicateGunGive":
        case "syndicateGunShoot":
        case "syndicateBackupAttack":
        case "syndicateChooseBackup":
            return "syndicate";
        case "chat":
        case "chatIsBlock":
        case "chatGroup":
        case "sendChat":
            return "chat";
        case "whisper":
        case "whisperToPlayer":
        case "sendWhisper":
            return "whisper";
        case "nominate":
        case "pitchforkVote":
        case "callWitness":
            return "vote";
        default:
            return id.type;
    }
}

function translateGroupName(id: ControllerID): string {
    switch (id.type) {
        case "role":
            return translate("role." + id.role + ".name");
        case "syndicateGunGive":
        case "syndicateGunShoot":
        case "syndicateBackupAttack":
        case "syndicateChooseBackup":
            return translate("mafia");
        case "nominate":
        case "pitchforkVote":
        case "callWitness":
            return translate("vote");
        default:
            return translateControllerID(id);
    }
}

/// True if this controller should be in this menu
export function controllerIsVisible(id: ControllerID, controller: SavedController): boolean {
    return (singleAbilityJsonData(controllerIdToLink(id))?.visible ?? true) && !controller.parameters.grayedOut;
}

export default function GenericAbilityMenu(): ReactElement {
    const savedAbilities = usePlayerState(
        (playerState) => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"],
    )!;

    const myIndex = usePlayerState((playerState) => playerState.myIndex, ["yourPlayerIndex"])!;

    const phaseState = useGameState((gameState) => gameState.phaseState, ["phase"])!;

    const controllerGroupsMap: ControllerGroupsMap = new ListMap();
    //build this map ^
    for (const [controllerID, controller] of savedAbilities) {
        if (
            !controllerIsVisible(controllerID, controller) &&
            !(
                // Special case: hide judge abilities unless on trial, the backend doesn't do this for us.
                controllerID.type === "judge" &&
                loadSettingsParsed().headerEnabled === false &&
                phaseState.type === "judgement" &&
                phaseState.playerOnTrial !== myIndex &&
                controllerID.player === myIndex &&
                !controller.parameters.grayedOut &&
                controller.parameters.allowedPlayers.includes(myIndex)
            )
        ) {
            continue;
        }
        const groupName = getGroupNameFromControllerID(controllerID);

        let controllers = controllerGroupsMap.get(groupName);
        if (controllers === null) {
            controllers = new ListMap(
                [],
                (k1, k2) => controllerIdToLinkWithPlayer(k1) === controllerIdToLinkWithPlayer(k2),
            );
        }

        controllers.insert(controllerID, controller);
        controllerGroupsMap.insert(groupName, controllers);
    }

    return (
        <>
            {controllerGroupsMap.entries().map(([group, controllers], i) => {
                return <MultipleControllersMenu key={i} groupName={group} controllers={controllers} />;
            })}
        </>
    );
}

function MultipleControllersMenu(
    props: Readonly<{
        groupName: string;
        controllers: ListMap<ControllerID, SavedController>;
    }>,
): ReactElement {
    if (props.controllers.entries().length === 0) {
        return <></>;
    }
    if (props.controllers.entries().length === 1) {
        const firstController = props.controllers.entries()[0];
        return <SingleAbilityMenu abilityId={firstController[0]} saveData={firstController[1]} key={0} />;
    }

    const disabled = !props.controllers.values().some((controller) => !controller.parameters.grayedOut);

    const anyControllerId = props.controllers.keys()[0];
    let groupName = "";
    if (anyControllerId !== undefined) {
        groupName = translateGroupName(anyControllerId);
    } else {
        return <></>;
    }

    return (
        <DetailsSummary
            className="generic-ability-menu"
            summary={
                <div className="generic-ability-menu-tab-summary">
                    <StyledText>{groupName}</StyledText>
                </div>
            }
            defaultOpen={true}
            disabled={disabled}
        >
            {props.controllers.entries().map(([id, saveData], i) => {
                return <SingleAbilityMenu key={i} abilityId={id} saveData={saveData} includeDropdown={false} />;
            })}
        </DetailsSummary>
    );
}

function SingleAbilityMenu(
    props: Readonly<{
        abilityId: ControllerID;
        key: number;
        saveData: SavedController;
        includeDropdown?: boolean;
    }>,
): ReactElement {
    const nightIcon = singleAbilityJsonData(controllerIdToLink(props.abilityId))?.midnight;
    const instantIcon = singleAbilityJsonData(controllerIdToLink(props.abilityId))?.instant;
    const visitIcon = singleAbilityJsonData(controllerIdToLink(props.abilityId))?.visit;

    let controllerIdName = translateControllerID(props.abilityId);
    if (props.abilityId.type === "role" && props.includeDropdown === false) {
        controllerIdName = translateControllerIDNoRole(props.abilityId) ?? "";
    }

    //The chat message makes it more verbose, showing more relevant information
    // as menus get large, it makes it harder to parse. so i keep it out for now
    const inner = (
        <>
            <SwitchSingleAbilityMenuType
                id={props.abilityId}
                available={props.saveData.parameters.available}
                selected={props.saveData.selection}
            />
        </>
    );

    if (props.includeDropdown === true || props.includeDropdown === undefined) {
        return (
            <DetailsSummary
                className="generic-ability-menu"
                summary={
                    <div className="generic-ability-menu-tab-summary">
                        <span>
                            <StyledText>{controllerIdName}</StyledText>
                        </span>
                        <span>
                            {instantIcon && <ControllerIcon icon={{ instant: true }} />}
                            {nightIcon && <ControllerIcon icon={{ night: true }} />}
                            {visitIcon && <ControllerIcon icon={{ visit: visitIcon }} />}
                        </span>
                    </div>
                }
                defaultOpen={true}
                disabled={props.saveData.parameters.grayedOut}
            >
                {inner}
            </DetailsSummary>
        );
    } else {
        return (
            <div className="generic-ability-menu generic-ability-menu-tab-no-summary">
                <div className="generic-no-summary">
                    <span>
                        <StyledText>{controllerIdName}</StyledText>
                    </span>
                    <span>
                        {instantIcon && <ControllerIcon icon={{ instant: true }} />}
                        {nightIcon && <ControllerIcon icon={{ night: true }} />}
                        {visitIcon && <ControllerIcon icon={{ visit: visitIcon }} />}
                    </span>
                </div>
                {inner}
            </div>
        );
    }
}

type ControllerIconData =
    | {
          instant: true;
      }
    | {
          night: true;
      }
    | {
          visit: "normal" | "abnormal" | "none";
      };

function ControllerIcon(props: Readonly<{ icon: ControllerIconData }>) {
    const anchorController = useContext(AnchorControllerContext)!;
    const menuController = useContext(MenuControllerContext)!;

    const children = useMemo(() => {
        if ("instant" in props.icon) {
            return translate("instant.icon");
        }
        if ("night" in props.icon) {
            return translate("night.icon");
        }
        if ("visit" in props.icon) {
            return translate("visit.icon." + props.icon.visit);
        }
        return null;
    }, [props.icon]);

    const tooltip = useMemo(() => {
        if ("instant" in props.icon) {
            return (
                <div className="wiki-article wiki-article-tooltip">
                    {translate("wiki.article.standard.controller.tooltip.instant")}
                </div>
            );
        }
        if ("night" in props.icon) {
            return (
                <div className="wiki-article wiki-article-tooltip">
                    {translate("wiki.article.standard.controller.tooltip.night")}
                </div>
            );
        }
        if ("visit" in props.icon) {
            return (
                <div className="wiki-article wiki-article-tooltip">
                    {translate(`wiki.article.standard.controller.tooltip.visit.${props.icon.visit}`)}
                </div>
            );
        }
    }, [props.icon]);

    return (
        <Button
            className="flush controller-icon"
            tooltip={tooltip}
            onClick={() => setWikiSearchPage("standard/controller", anchorController, menuController)}
        >
            {children}
        </Button>
    );
}

function SwitchSingleAbilityMenuType(
    props: Readonly<{
        id: ControllerID;
        available: AvailableControllerSelection;
        selected: ControllerSelection;
    }>,
): ReactElement {
    const { id, available } = props;
    const selected: ControllerSelection = props.selected;

    switch (available.type) {
        case "chatGroup":
            return (
                <ChatGroupSelect
                    id={id}
                    groups={available.selection.groups}
                    selected={selected.type === "chatGroup" ? selected.selection : null}
                />
            );
        case "unit":
            return (
                <Button
                    onClick={() => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: { type: "unit", selection: null },
                        });
                    }}
                >
                    {translateControllerID(props.id)}
                </Button>
            );
        case "boolean": {
            let bool;
            if (selected === null || selected.type !== "boolean") {
                bool = false;
            } else {
                bool = selected.selection;
            }

            return (
                <BooleanSelectionMenu
                    id={id}
                    selection={bool}
                    onChoose={(x) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "boolean",
                                selection: x,
                            },
                        });
                    }}
                />
            );
        }
        case "playerList": {
            let input: PlayerListSelection;
            if (props.selected === null || props.selected.type !== "playerList") {
                input = [];
            } else {
                input = props.selected.selection;
            }

            return (
                <PlayerListSelectionMenu
                    selection={input}
                    availableSelection={available.selection}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "playerList",
                                selection,
                            },
                        });
                    }}
                />
            );
        }
        case "twoPlayerOption": {
            let input: TwoPlayerOptionSelection;
            if (props.selected === null || props.selected.type !== "twoPlayerOption") {
                input = null;
            } else {
                input = props.selected.selection;
            }

            return (
                <TwoPlayerOptionSelectionMenu
                    selection={input}
                    availableSelection={available.selection}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "twoPlayerOption",
                                selection,
                            },
                        });
                    }}
                />
            );
        }
        case "roleList": {
            let input: RoleListSelection;
            if (props.selected === null || props.selected.type !== "roleList") {
                input = [];
            } else {
                input = props.selected.selection;
            }

            return (
                <RoleListSelectionMenu
                    selection={input}
                    availableSelection={available.selection}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "roleList",
                                selection,
                            },
                        });
                    }}
                />
            );
        }
        case "twoRoleOption": {
            let input: TwoRoleOptionSelection;
            if (props.selected === null || props.selected.type !== "twoRoleOption") {
                input = [null, null];
            } else {
                input = props.selected.selection;
            }

            return (
                <TwoRoleOptionSelectionMenu
                    input={input}
                    availableSelection={available.selection}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "twoRoleOption",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        case "graveDeathCauses": {
            let input: GraveDeathCausesSelection;
            if (props.selected === null || props.selected.type !== "graveDeathCauses") {
                input = [];
            } else {
                input = props.selected.selection;
            }

            return (
                <GraveDeathCausesSelectionMenu
                    selection={input}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "graveDeathCauses",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        case "twoRoleOutlineOption": {
            let input: TwoRoleOutlineOptionSelection;
            if (props.selected === null || props.selected.type !== "twoRoleOutlineOption") {
                input = [null, null];
            } else {
                input = props.selected.selection;
            }

            return (
                <TwoRoleOutlineOptionSelectionMenu
                    selection={input}
                    available={available.selection}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "twoRoleOutlineOption",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        case "string": {
            let input: StringSelection;
            if (props.selected === null || props.selected.type !== "string") {
                input = "";
            } else {
                input = props.selected.selection;
            }

            return (
                <StringSelectionMenu
                    id={id}
                    selection={input}
                    onChoose={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "string",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        case "integer": {
            let input: IntegerSelection;
            if (props.selected === null || props.selected.type !== "integer") {
                input = 0;
            } else {
                input = props.selected.selection;
            }

            return (
                <IntegerSelectionMenu
                    id={id}
                    selection={input}
                    available={available.selection}
                    onChoose={(selection: number) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "integer",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        case "kira": {
            let input: KiraSelection;
            if (props.selected === null || props.selected.type !== "kira") {
                input = [];
            } else {
                input = props.selected.selection;
            }

            return (
                <KiraSelectionMenu
                    selection={input}
                    available={available.selection}
                    onChange={(selection) => {
                        GAME_MANAGER.sendControllerInput({
                            id,
                            selection: {
                                type: "kira",
                                selection: selection,
                            },
                        });
                    }}
                />
            );
        }
        default:
            return <></>;
    }
}
