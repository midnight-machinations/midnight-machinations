import React, { ReactElement, useContext, useEffect, useMemo, useState } from "react";
import { DEV_ENV } from "../../index";
import LobbyPlayerList from "./LobbyPlayerList";
import "./lobbyMenu.css";
import translate from "../../game/lang";
import { RoomLinkButton } from "../GlobalMenu";
import { RoleList, getAllRoles } from "../../stateContext/stateType/roleListState";
import LoadingScreen from "../LoadingScreen";
import { GameModeContext } from "../../components/gameModeSettings/GameModesEditor";
import PhaseTimesSelector from "../../components/gameModeSettings/PhaseTimeSelector";
import { OutlineListSelector } from "../../components/gameModeSettings/OutlineSelector";
import EnabledRoleSelector from "../../components/gameModeSettings/EnabledRoleSelector";
import Icon from "../../components/Icon";
import { GameModeSelector } from "../../components/gameModeSettings/GameModeSelector";
import LobbyChatMenu from "./LobbyChatMenu";
import { Button } from "../../components/Button";
import { EnabledModifiersSelector } from "../../components/gameModeSettings/EnabledModifiersSelector";
import LobbyNamePane from "./LobbyNamePane";
import { MobileContext } from "../MobileContext";
import { WebsocketContext } from "../WebsocketContext";
import { AppContext } from "../AppContext";
import { StateContext } from "../../stateContext/StateContext";

export default function LobbyMenu(props: Readonly<{}>): ReactElement {
    const websocketContext = useContext(WebsocketContext)!;
    const [loading, setLoading] = useState<boolean>(true);

    // This doesn't catch all the packets :(
    useEffect(() => {
        console.log(websocketContext.lastMessageRecieved?.type);
        if (websocketContext.lastMessageRecieved?.type === "lobbyName") {
            setLoading(false);
        }
    }, [websocketContext.lastMessageRecieved]);

    return loading ? <LoadingScreen type="join" /> : <LobbyMenuInner/>
}

function LobbyMenuInner(): ReactElement {
    const lobbyState = useContext(StateContext)!;

    const isSpectator = lobbyState.clients.get(lobbyState.myId!)?.clientType.type === "spectator";

    const myClient = lobbyState.clients.get(lobbyState.myId!);
    let isHost = true;
    if (myClient !== null){
        isHost = myClient.ready === "host";
    }
    const mobile = useContext(MobileContext)!;

    const [advancedView, setAdvancedView] = useState<boolean>(isHost || mobile);

    useEffect(() => {
        setAdvancedView(isHost || mobile)
    }, [mobile, isHost])

    useEffect(() => {
        const onBeforeUnload = (e: BeforeUnloadEvent) => {
            if (!DEV_ENV) e.preventDefault()
        };

        window.addEventListener("beforeunload", onBeforeUnload);
        return () => window.removeEventListener("beforeunload", onBeforeUnload);
    }, [])

    return <div className="lm">
        <div>
            <LobbyMenuHeader isHost={isHost} advancedView={advancedView} setAdvancedView={setAdvancedView}/>
            {advancedView 
                ? <main>
                    <div>
                        <LobbyNamePane/>
                        <LobbyPlayerList/>
                        <LobbyChatMenu spectator={isSpectator}/>
                    </div>
                    <div>
                        <LobbyMenuSettings isHost={isHost}/>
                    </div>
                </main>
                : <main>
                    <div>
                        <LobbyNamePane/>
                        <LobbyPlayerList/>
                    </div>
                    <div>
                        <LobbyChatMenu spectator={isSpectator}/>
                    </div>
                </main>
            }
        </div>
    </div>
}

function LobbyMenuSettings(props: Readonly<{
    isHost: boolean,
}>): JSX.Element {
    const lobbyState = useContext(StateContext)!;
    const roleList = lobbyState.roleList;
    const enabledRoles = lobbyState.enabledRoles;
    const phaseTimes = lobbyState.phaseTimes;
    const enabledModifiers = lobbyState.enabledModifiers;

    const mobile = useContext(MobileContext)!;
    const {
        sendSetPhaseTimesPacket, sendEnabledRolesPacket, sendSetRoleListPacket, sendEnabledModifiersPacket, sendSetRoleOutlinePacket
    } = useContext(WebsocketContext)!;

    const sendRoleList = (newRoleList: RoleList) => {
        const combinedRoleList = structuredClone(roleList);
        newRoleList.forEach((role, index) => {
            combinedRoleList[index] = role
        })
        sendSetRoleListPacket(combinedRoleList);
    };

    const context = useMemo(() => {
        return {roleList, enabledRoles, phaseTimes, enabledModifiers};
    }, [enabledRoles, phaseTimes, roleList, enabledModifiers]);

    return <GameModeContext.Provider value={context}>
        {mobile && <h1>{translate("menu.lobby.settings")}</h1>}
        {props.isHost === true && <GameModeSelector 
            canModifySavedGameModes={false}
            loadGameMode={gameMode => {
                sendSetPhaseTimesPacket(gameMode.phaseTimes);
                sendEnabledRolesPacket(gameMode.enabledRoles);
                sendSetRoleListPacket(gameMode.roleList);
                sendEnabledModifiersPacket(gameMode.enabledModifiers);
            }}
        />}
        <EnabledModifiersSelector
            disabled={!props.isHost}
            onChange={modifiers => sendEnabledModifiersPacket(modifiers)}
        />
        <PhaseTimesSelector 
            disabled={!props.isHost}
            onChange={pts => sendSetPhaseTimesPacket(pts)}
        />
        <OutlineListSelector
            disabled={!props.isHost}
            onChangeRolePicker={(value, index) => sendSetRoleOutlinePacket(index, value)}
            onAddNewOutline={undefined}
            onRemoveOutline={undefined}
            setRoleList={sendRoleList}
        />
        <EnabledRoleSelector
            onEnableRoles={roles => sendEnabledRolesPacket([...enabledRoles, ...roles])}
            onDisableRoles={roles => sendEnabledRolesPacket(enabledRoles.filter(role => !roles.includes(role)))}
            onIncludeAll={() => sendEnabledRolesPacket(getAllRoles())}
            disabled={!props.isHost}
        />
    </GameModeContext.Provider>
}

// There's probably a better way to do this that doesn't need the mobile check.
function LobbyMenuHeader(props: Readonly<{
    isHost: boolean,
    advancedView: boolean,
    setAdvancedView: (advancedView: boolean) => void
}>): JSX.Element {
    const { lobbyName } = useContext(StateContext)!;
    const { sendStartGamePacket, sendSetLobbyNamePacket } = useContext(WebsocketContext)!;
    const mobile = useContext(MobileContext)!;
    const { setContent: setAnchorContent } = useContext(AppContext)!;
    
    const [localLobbyName, setLobbyName] = useState<string>(lobbyName ?? "Mafia Lobby");
    
    return <header>
        <div>
            <Button disabled={!props.isHost} className="start" onClick={async ()=>{
                sendStartGamePacket()
                setAnchorContent(<LoadingScreen type="default"/>);
                if (await sendStartGamePacket()) {
                    // TODO HARDCODED FOR NOW
                    setAnchorContent({ type: "gameScreen", spectator: false })
                }
            }}>
                <Icon>play_arrow</Icon>{translate("menu.lobby.button.start")}
            </Button>
            <RoomLinkButton/>
            {mobile || props.isHost || <Button
                onClick={() => props.setAdvancedView(!props.advancedView)}
            >
                <Icon>settings</Icon>{translate(`menu.lobby.button.advanced.${props.advancedView}`)}
            </Button>}
        </div>
        {props.isHost ? 
            <input 
                type="text" 
                value={lobbyName??"Unnamed Lobby"}
                onInput={e => {
                    setLobbyName((e.target as HTMLInputElement).value);
                }}
                onKeyUp={(e)=>{
                    if(e.key !== 'Enter') return;
                    
                    const newLobbyName = (e.target as HTMLInputElement).value;
                    setLobbyName(newLobbyName);
                    sendSetLobbyNamePacket(newLobbyName);
                    
                }}
                onBlur={e => {
                    const newLobbyName = (e.target as HTMLInputElement).value;
                    setLobbyName(newLobbyName);
                    sendSetLobbyNamePacket(newLobbyName);
                }}
            /> : 
            <h3>{lobbyName}</h3>
        }
        
    </header>
}

