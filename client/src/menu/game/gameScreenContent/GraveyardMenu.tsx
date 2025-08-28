import React, { ReactElement } from "react";
import translate from "../../../game/lang";
import GAME_MANAGER from "../../../index";
import { ContentMenu, ContentTab } from "../GameScreen";
import "./graveyardMenu.css";
import StyledText from "../../../components/StyledText";
import { EnabledRolesDisplay } from "../../../components/gameModeSettings/EnabledRoleSelector";
import { useGameState, useLobbyOrGameState, usePlayerState, useSpectator } from "../../../components/useHooks";
import { translateRoleOutline } from "../../../game/roleListState.d";
import { Button } from "../../../components/Button";
import DetailsSummary from "../../../components/DetailsSummary";
import { ModifierSettingsDisplay } from "../../../components/gameModeSettings/ModifiersSelector";

export default function GraveyardMenu(): ReactElement {
    return <div className="graveyard-menu graveyard-menu-colors">
        <ContentTab close={ContentMenu.GraveyardMenu} helpMenu={"standard/gameMode"}>{translate("menu.gameMode.title")}</ContentTab>
            
        <DetailsSummary
            summary={translate("menu.lobby.roleList")}
            defaultOpen={true}
        >
            <RoleListDisplay />
        </DetailsSummary>
        <EnabledRoles/>
        <EnabledModifiers/>
    </div>
}

function RoleListDisplay(): ReactElement {
    const roleList = useLobbyOrGameState(
        gameState => gameState.roleList,
        ["roleList"]
    )!
    const crossedOutOutlines = usePlayerState(
        clientState => clientState.crossedOutOutlines,
        ["yourCrossedOutOutlines"],
        []
    )!
    const playerNames = useGameState(
        gameState => gameState.players.map(player => player.toString()),
        ["gamePlayers"]
    )!

    const spectator = useSpectator();

    return <>
        {roleList.map((entry, index)=>{
            return <Button
                className="role-list-button"
                style={{ gridRow: index + 1}}
                key={index}
                onClick={()=>{
                    if (spectator) return;

                    let newCrossedOutOutlines;
                    if(crossedOutOutlines.includes(index))
                        newCrossedOutOutlines = crossedOutOutlines.filter(x=>x!==index);
                    else
                        newCrossedOutOutlines = crossedOutOutlines.concat(index);

                    GAME_MANAGER.sendSaveCrossedOutOutlinesPacket(newCrossedOutOutlines);
                }}
            >
                {
                    crossedOutOutlines.includes(index) ? 
                    <s><StyledText>
                        {`${index + 1}: ` + translateRoleOutline(entry, playerNames)}
                    </StyledText></s> : 
                    <StyledText>
                        {`${index + 1}: ` + translateRoleOutline(entry, playerNames)}
                    </StyledText>
                }
            </Button>
        })}
    </>
}

function EnabledRoles(): ReactElement {
    const enabledRoles = useGameState(
        gameState => gameState.enabledRoles,
        ["enabledRoles"]
    )!

    return <div className="graveyard-menu-excludedRoles">
        <DetailsSummary
            summary={translate("menu.enabledRoles.enabledRoles")}
        >
            <EnabledRolesDisplay enabledRoles={enabledRoles}/>
        </DetailsSummary>
    </div>
}

function EnabledModifiers(): ReactElement {
    const modifierSettings = useGameState(
        gameState=>gameState.modifierSettings.list,
        ["modifierSettings"]
    )!

    return <div className="graveyard-menu-excludedRoles">
        <DetailsSummary
            summary={translate("modifiers")}
        >
            <ModifierSettingsDisplay disabled={true} modifierSettings={modifierSettings}/>
        </DetailsSummary>
    </div>
}