import { ReactElement } from "react";
import { 
    TwoPlayerOptionSelection, 
    TwoRoleOptionSelection, 
    ControllerID,
    ControllerSelection,
    translateControllerID,
    AvailableControllerSelection,
    TwoRoleOutlineOptionSelection,
    RoleListSelection,
    SavedController,
    controllerIdToLinkWithPlayer,
    singleAbilityJsonData,
    StringSelection,
    translateControllerIDNoRole,
    PlayerListSelection,
    IntegerSelection,
    controllerIdToLink
} from "../../../../game/controllerInput";
import React from "react";
import { usePlayerState } from "../../../../components/useHooks";
import { Button } from "../../../../components/Button";
import TwoRoleOutlineOptionSelectionMenu from "./ControllerSelectionTypes/TwoRoleOutlineOptionSelectionMenu";
import GAME_MANAGER from "../../../..";
import TwoRoleOptionSelectionMenu from "./ControllerSelectionTypes/TwoRoleOptionSelectionMenu";
import TwoPlayerOptionSelectionMenu from "./ControllerSelectionTypes/TwoPlayerOptionSelectionMenu";
import StyledText from "../../../../components/StyledText";
import KiraSelectionMenu, { KiraSelection } from "./ControllerSelectionTypes/KiraSelectionMenu";
import RoleListSelectionMenu from "./ControllerSelectionTypes/RoleListSelectionMenu";
import "./genericAbilityMenu.css";
import DetailsSummary from "../../../../components/DetailsSummary";
import translate from "../../../../game/lang";
import StringSelectionMenu from "./ControllerSelectionTypes/StringSelectionMenu";
import ListMap from "../../../../ListMap";
import { Role } from "../../../../game/roleState.d";
import { PlayerIndex } from "../../../../game/gameState.d";
import Icon from "../../../../components/Icon";
import PlayerListSelectionMenu from "./ControllerSelectionTypes/PlayerListSelectionMenu";
import IntegerSelectionMenu from "./ControllerSelectionTypes/IntegerSelectionMenu";
import BooleanSelectionMenu from "./ControllerSelectionTypes/BooleanSelectionMenu";

type GroupName = `${PlayerIndex}/${Role}` | 
    "syndicateGunItem" | 
    "backup" | 
    "chat" |
    "whisper" |
    ControllerID["type"];

type ControllerGroupsMap = ListMap<
    GroupName, 
    ListMap<ControllerID, SavedController>
>;

function getGroupNameFromControllerID(id: ControllerID): GroupName {
    switch (id.type){
        case "role":
            return id.player+"/"+id.role as `${PlayerIndex}/${Role}`
        case "syndicateGunItemGive":
        case "syndicateGunItemShoot":
            return "syndicateGunItem";
        case "syndicateBackupAttack":
        case "syndicateChooseBackup":
            return "backup";
        case "chat":
        case "chatIsBlock":
        case "sendChat":
            return "chat";
        case "whisper":
        case "whisperToPlayer":
        case "sendWhisper":
            return "whisper";
        default:
            return id.type;
    }
}

function translateGroupName(id: ControllerID): string {
    switch (id.type){
        case "role":
            return translate("role."+id.role+".name");
        case "syndicateGunItemGive":
        case "syndicateGunItemShoot":
            return translate("syndicateGunItem");
        case "syndicateBackupAttack":
        case "syndicateChooseBackup":
            return translate("backup");
        default:
            return id.type;
    }
}

/// True if this controller should be in this menu
function showThisController(id: ControllerID): boolean {
    return (singleAbilityJsonData(controllerIdToLink(id))?.visible)??true;
}

export default function GenericAbilityMenu(): ReactElement {
    const savedAbilities = usePlayerState(
        playerState => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"]
    )!;

    let controllerGroupsMap: ControllerGroupsMap = new ListMap();
    //build this map ^
    for(let [controllerID, controller] of savedAbilities) {

        if (!showThisController(controllerID)) {continue;}

        let groupName = getGroupNameFromControllerID(controllerID);
        
        let controllers = controllerGroupsMap.get(groupName);
        if(controllers === null){
            controllers = new ListMap([], (k1, k2)=>controllerIdToLinkWithPlayer(k1)===controllerIdToLinkWithPlayer(k2));
        }

        controllers.insert(controllerID, controller);
        controllerGroupsMap.insert(groupName, controllers);
    }

    return <>
        {controllerGroupsMap.entries().map(([group, controllers], i)=>{

            let firstController = controllers.entries()[0]
            if(firstController !== undefined && controllers.entries().length === 1){
                return <SingleAbilityMenu
                    key={i}
                    abilityId={firstController[0]}
                    saveData={firstController[1]}
                />
            }else{
                return <MultipleControllersMenu
                    key={i}
                    groupName={group}
                    controllers={controllers}
                />
            }
        })}
    </>
}

function MultipleControllersMenu(props: Readonly<{
    groupName: string,
    controllers: ListMap<ControllerID, SavedController>
}>): ReactElement {

    const disabled = !props.controllers.values().some((controller)=>!controller.parameters.grayedOut)
    const nightIcon = !props.controllers.keys().some(
        (id)=>!singleAbilityJsonData(controllerIdToLink(id))?.midnight
    );


    let anyControllerId = props.controllers.keys()[0]
    let groupName = "";
    if(anyControllerId !== undefined){
        groupName = translateGroupName(anyControllerId)
    }else{
        return <></>;
    }

    return <DetailsSummary
        className="generic-ability-menu"
        summary={
            <div className="generic-ability-menu-tab-summary">
                <StyledText>{groupName}</StyledText>
                {nightIcon?<span>{translate("night.icon")}</span>:null}
            </div>
        }
        defaultOpen={true}
        disabled={disabled}
    >
        {props.controllers.entries().map(([id, saveData], i) => {
            return <SingleAbilityMenu
                key={i}
                abilityId={id}
                saveData={saveData}
                includeDropdown={false}
            />
        })}
    </DetailsSummary>
}

function SingleAbilityMenu(props: Readonly<{
    abilityId: ControllerID,
    key: number,
    saveData: SavedController,
    includeDropdown?: boolean
}>): ReactElement {
    const nightIcon = singleAbilityJsonData(controllerIdToLink(props.abilityId))?.midnight;

    let controllerIdName = translateControllerID(props.abilityId);
    if(props.abilityId.type === "role" && props.includeDropdown === false){
        controllerIdName = (translateControllerIDNoRole(props.abilityId)??"");
    }

    //The chat message makes it more verbose, showing more relevant information
    // as menus get large, it makes it harder to parse. so i keep it out for now
    const inner = <>
        {/* {props.saveData.availableAbilityData.dontSave ? null : 
            <ChatMessage message={{
                variant: {
                    type: "abilityUsed",
                    player: myIndex,
                    abilityId: props.abilityId,
                    selection: props.saveData.selection
                },
                chatGroup: "all"
            }}/>
        } */}
        <SwitchSingleAbilityMenuType
            id={props.abilityId}
            available={props.saveData.parameters.available}
            selected={props.saveData.selection}
        />
    </>

    if(props.includeDropdown===true || props.includeDropdown===undefined){
        return <DetailsSummary
            className="generic-ability-menu"
            summary={
                <div className="generic-ability-menu-tab-summary">
                    <span><StyledText>{controllerIdName}</StyledText></span>
                    {nightIcon?<span>{translate("night.icon")}</span>:null}
                </div>
            }
            defaultOpen={true}
            disabled={props.saveData.parameters.grayedOut}
        >
            {inner}
        </DetailsSummary>
        
    }else{
        return <>
            <div className="generic-ability-menu generic-ability-menu-tab-no-summary">
                <span>
                    {
                        props.saveData.parameters.grayedOut === true ?
                        <Icon>close</Icon>
                        : null
                    }
                    <StyledText>{controllerIdName}</StyledText>
                </span>
                {nightIcon?<span>{translate("night.icon")}</span>:null}
            </div>
            {
                props.saveData.parameters.grayedOut === false ?
                <>{inner}</>
                : null
            }
        </>
    }
    
}


function SwitchSingleAbilityMenuType(props: Readonly<{
    id: ControllerID,
    available: AvailableControllerSelection,
    selected: ControllerSelection
}>): ReactElement {

    const {id, available} = props;
    let selected: ControllerSelection = props.selected;

    switch(available.type) {
        case "unit":
            return <Button
                onClick={()=>{
                    GAME_MANAGER.sendControllerInput({
                        id, 
                        selection: {type: "unit", selection: null}
                    });
                }}
            >
                {translateControllerID(props.id)}
            </Button>
        case "boolean":{
            let bool;
            if(selected === null || selected.type !== "boolean"){
                bool = false;
            }else{
                bool = selected.selection;
            }

            return <BooleanSelectionMenu
                id={id}
                selection={bool}
                onChoose={(x)=>{
                    GAME_MANAGER.sendControllerInput({
                        id, 
                        selection: {
                            type: "boolean",
                            selection: x
                        }
                    });
                }}
            />;
        }
        case "playerList":{
            let input: PlayerListSelection;
            if(
                props.selected === null ||
                props.selected.type !== "playerList"
            ){
                input = [];
            }else{
                input = props.selected.selection;
            }

            return <PlayerListSelectionMenu
                selection={input}
                availableSelection={available.selection}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id, 
                        selection: {
                            type: "playerList",
                            selection
                        }
                    });
                }}
            />;
        }
        case "twoPlayerOption":{
            let input: TwoPlayerOptionSelection;
            if(
                props.selected === null ||
                props.selected.type !== "twoPlayerOption"
            ){
                input = null;
            }else{
                input = props.selected.selection;
            }

            return <TwoPlayerOptionSelectionMenu
                selection={input}
                availableSelection={available.selection}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id, 
                        selection: {
                            type: "twoPlayerOption",
                            selection
                        }
                    });
                }}
            />;
        }
        case "roleList":{
            let input: RoleListSelection;
            if(
                props.selected === null ||
                props.selected.type !== "roleList"
            ){
                input = [];
            }else{
                input = props.selected.selection;
            }

            return <RoleListSelectionMenu
                selection={input}
                availableSelection={available.selection}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id, 
                        selection: {
                            type: "roleList",
                            selection
                        }
                    });
                }}
            />
        }
        case "twoRoleOption":{

            let input: TwoRoleOptionSelection;
            if(
                props.selected === null ||
                props.selected.type !== "twoRoleOption"
            ){
                input = [null, null];
            }else{
                input = props.selected.selection;
            }

            return <TwoRoleOptionSelectionMenu
                input={input}
                availableSelection={available.selection}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id,
                        selection: {
                            type: "twoRoleOption",
                            selection: selection
                        }
                    });
                }}
            />;
        }
        case "twoRoleOutlineOption":{
            let input: TwoRoleOutlineOptionSelection;
            if(
                props.selected === null ||
                props.selected.type !== "twoRoleOutlineOption"
            ){
                input = [null, null];
            }else{
                input = props.selected.selection;
            }

            return <TwoRoleOutlineOptionSelectionMenu
                selection={input}
                available={available.selection}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id,
                        selection: {
                            type: "twoRoleOutlineOption",
                            selection: selection
                        }
                    });
                }}
            />
        }
        case "string":{
            let input: StringSelection;
            if(
                props.selected === null ||
                props.selected.type !== "string"
            ){
                input = "";
            }else{
                input = props.selected.selection;
            }

            return <StringSelectionMenu
                id={id}
                selection={input}
                onChoose={(selection) => {
                    GAME_MANAGER.sendControllerInput({
                        id,
                        selection: {
                            type: "string",
                            selection: selection
                        }
                    });
                }}
            />
        }
        case "integer":{
            let input: IntegerSelection;
            if(
                props.selected === null ||
                props.selected.type !== "integer"
            ){
                input = 0;
            }else{
                input = props.selected.selection;
            }

            return <IntegerSelectionMenu
                id={id}
                selection={input}
                available={available.selection}
                onChoose={(selection: number) => {
                    GAME_MANAGER.sendControllerInput({
                        id,
                        selection: {
                            type: "integer",
                            selection: selection
                        }
                    });
                }}
            />
        }
        case "kira":{
            let input: KiraSelection;
            if(
                props.selected === null ||
                props.selected.type !== "kira"
            ){
                input = [];
            }else{
                input = props.selected.selection;
            }

            return <KiraSelectionMenu
                selection={input}
                available={available.selection}
                onChange={(selection)=>{
                    GAME_MANAGER.sendControllerInput({
                        id,
                        selection: {
                            type: "kira",
                            selection: selection
                        }
                    });
                }}
            />
        }
        default:
            return <></>;
    }
}
