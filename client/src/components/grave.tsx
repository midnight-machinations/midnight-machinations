import { type ReactElement, useMemo } from "react";
import { replaceMentions } from "..";
import { type Grave, type GraveInformation, translateGraveDeathCauses } from "../game/graveState";
import translate from "../game/lang";
import { encodeString } from "./ChatMessage";
import StyledText from "./StyledText";
import "./grave.css";
import type { UnsafeString } from "../game/gameState.d";
import type { RoleList } from "../game/roleListState.d";
import { useGameState, useLobbyOrGameState } from "./useHooks";
import { buttonProps } from "./keyboard";

export function translateGraveRole(grave: Grave): string {
    if (grave.information.type === "obscured") {
        return translate("obscured");
    } else {
        return translate(`role.${grave.information.role}.name`);
    }
}

export default function GraveComponent(
    props: Readonly<{
        grave: Grave;
        playerNames?: UnsafeString[];
        roleList?: RoleList;
        onClick?: () => void;
        noLinks?: boolean;
    }>,
): ReactElement {
    const gamePlayerNames = useGameState(
        (gameState) => gameState.players.map((player) => encodeString(player.toString())),
        ["gamePlayers"],
    )!;

    const playerNames = props.playerNames ?? gamePlayerNames;

    const gameRoleList = useLobbyOrGameState((gameState) => gameState.roleList, ["roleList"])!;

    const roleList = props.roleList ?? gameRoleList;

    if (props.grave.information.type === "obscured") {
        return <ObscuredGrave noLinks={props.noLinks} grave={props.grave} playerNames={playerNames} />;
    } else {
        return (
            <UnobscuredGrave
                noLinks={props.noLinks}
                grave={props.grave as any}
                playerNames={playerNames}
                roleList={roleList}
            />
        );
    }
}

function UnobscuredGrave(
    props: Readonly<{
        grave: Grave & { information: GraveInformation & { type: "normal" } };
        playerNames: UnsafeString[];
        roleList: RoleList;
        onClick?: () => void;
        noLinks?: boolean;
    }>,
): ReactElement {
    const graveDeathCause = useMemo(() => {
        return translateGraveDeathCauses(props.grave.information.deathCauses);
    }, [props.grave.information.deathCauses]);

    const graveRoleString = useMemo(() => {
        return translate(`role.${props.grave.information.role}.name`);
    }, [props.grave.information.role]);

    const diedPhaseString = useMemo(() => {
        return props.grave.diedPhase === "day" ? translate("day") : translate("phase.night");
    }, [props.grave.diedPhase]);

    const diedPhaseIcon = useMemo(() => {
        return props.grave.diedPhase === "day" ? translate("day.icon") : translate("night.icon");
    }, [props.grave.diedPhase]);

    return (
        <div
            className="grave graveyard-menu-colors"
            {...(props.onClick !== undefined ? buttonProps(props.onClick) : {})}
        >
            <div>
                <StyledText
                    noLinks={props.noLinks}
                >{`${diedPhaseString + diedPhaseIcon + props.grave.dayNumber}`}</StyledText>
            </div>
            <div>
                <StyledText
                    noLinks={props.noLinks}
                >{`${`${props.playerNames[props.grave.player]} (${graveRoleString})`}`}</StyledText>
            </div>
            {graveDeathCause && (
                <div>
                    <StyledText
                        noLinks={props.noLinks}
                    >{`${`${translate("killedBy")} ${graveDeathCause}`}`}</StyledText>
                </div>
            )}
            {(props.grave.information.alibi as string).length === 0 || (
                <>
                    {translate("alibi")}
                    <div className="note-area">
                        <StyledText noLinks={props.noLinks}>
                            {encodeString(
                                replaceMentions(props.grave.information.alibi, props.playerNames, props.roleList),
                            )}
                        </StyledText>
                    </div>
                </>
            )}
            {props.grave.information.callingCards.length === 0 ||
                props.grave.information.callingCards.map((note) => (
                    <>
                        {translate("grave.callingCard")}
                        <div className="note-area">
                            <StyledText noLinks={props.noLinks}>
                                {encodeString(replaceMentions(note, props.playerNames, props.roleList))}
                            </StyledText>
                        </div>
                    </>
                ))}
        </div>
    );
}

function ObscuredGrave(
    props: Readonly<{ noLinks?: boolean; grave: Grave; playerNames: UnsafeString[] }>,
): ReactElement {
    const diedPhaseString = props.grave.diedPhase === "day" ? translate("day") : translate("phase.night");
    const diedPhaseIcon = props.grave.diedPhase === "day" ? translate("day.icon") : translate("night.icon");
    const graveRoleString = translate("obscured");

    return (
        <div className="grave graveyard-menu-colors">
            <div>
                <StyledText
                    noLinks={props.noLinks}
                >{`${diedPhaseString + diedPhaseIcon + props.grave.dayNumber}`}</StyledText>
            </div>
            <div>
                <StyledText
                    noLinks={props.noLinks}
                >{`${`${props.playerNames[props.grave.player]} (${graveRoleString})`}`}</StyledText>
            </div>
        </div>
    );
}
