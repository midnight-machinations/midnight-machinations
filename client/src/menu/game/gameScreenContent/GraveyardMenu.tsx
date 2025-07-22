import React, { ReactElement, useContext } from "react";
import translate from "../../../game/lang";
import "./graveyardMenu.css";
import StyledText from "../../../components/StyledText";
import { EnabledRolesDisplay } from "../../../components/gameModeSettings/EnabledRoleSelector";
import { translateRoleOutline } from "../../../stateContext/stateType/roleListState";
import { Button } from "../../../components/Button";
import DetailsSummary from "../../../components/DetailsSummary";
import { EnabledModifiersDisplay } from "../../../components/gameModeSettings/EnabledModifiersSelector";
import { GameScreenMenuType } from "../GameScreenMenuContext";
import GameScreenMenuTab from "../GameScreenMenuTab";
import { WebsocketContext } from "../../WebsocketContext";
import { usePlayerState } from "../../../stateContext/useHooks";
import { StateContext } from "../../../stateContext/StateContext";

export default function GraveyardMenu(): ReactElement {
    return <div className="graveyard-menu graveyard-menu-colors">
        <GameScreenMenuTab close={GameScreenMenuType.GraveyardMenu} helpMenu={"standard/gameMode"}>{translate("menu.gameMode.title")}</GameScreenMenuTab>
            
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
    const roleList = useContext(StateContext)!.roleList;
    const playerState = usePlayerState();
    const crossedOutOutlines = playerState!==undefined?playerState.crossedOutOutlines:[];

    const spectator = useContext(StateContext)!.clientState.type === "spectator";
    
    const websocketContext = useContext(WebsocketContext)!;

    return <>
        {roleList.map((entry, index)=>{
            return <Button
                className="role-list-button placard"
                style={{ gridRow: index + 1 }}
                key={index}
                onClick={()=>{
                    if (spectator) return;

                    let newCrossedOutOutlines;
                    if(crossedOutOutlines.includes(index))
                        newCrossedOutOutlines = crossedOutOutlines.filter(x=>x!==index);
                    else
                        newCrossedOutOutlines = crossedOutOutlines.concat(index);

                    websocketContext.sendSaveCrossedOutOutlinesPacket(newCrossedOutOutlines);
                }}
            >
                {
                    crossedOutOutlines.includes(index) ? 
                    <s><StyledText>
                        {translateRoleOutline(entry)}
                    </StyledText></s> : 
                    <StyledText>
                        {translateRoleOutline(entry)}
                    </StyledText>
                }
            </Button>
        })}
    </>
}

function EnabledRoles(): ReactElement {
    const enabledRoles = useContext(StateContext)!.enabledRoles;

    return <div className="graveyard-menu-excludedRoles">
        <DetailsSummary
            summary={translate("menu.enabledRoles.enabledRoles")}
        >
            <EnabledRolesDisplay enabledRoles={enabledRoles}/>
        </DetailsSummary>
    </div>
}

function EnabledModifiers(): ReactElement {
    const enabledModifiers = useContext(StateContext)!.enabledModifiers;

    return <div className="graveyard-menu-excludedRoles">
        <DetailsSummary
            summary={translate("modifiers")}
        >
            <EnabledModifiersDisplay disabled={true} enabledModifiers={enabledModifiers}/>
        </DetailsSummary>
    </div>
}