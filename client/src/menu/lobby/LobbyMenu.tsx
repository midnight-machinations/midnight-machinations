import React, { ReactElement, useContext, useEffect, useMemo, useState } from "react";
import GAME_MANAGER, { DEV_ENV } from "../../index";
import LobbyPlayerList from "./LobbyPlayerList";
import "./lobbyMenu.css";
import translate from "../../game/lang";
import { StateListener } from "../../game/gameManager.d";
import { AnchorControllerContext, MobileContext } from "../Anchor";
import { RoomLinkButton } from "../GlobalMenu";
import { RoleList, getAllRoles } from "../../game/roleListState.d";
import LoadingScreen from "../LoadingScreen";
import StartMenu from "../main/StartMenu";
import { GameModeContext } from "../../components/gameModeSettings/GameModesEditor";
import PhaseTimesSelector from "../../components/gameModeSettings/PhaseTimeSelector";
import { OutlineListSelector } from "../../components/gameModeSettings/OutlineSelector";
import EnabledRoleSelector from "../../components/gameModeSettings/EnabledRoleSelector";
import Icon from "../../components/Icon";
import { GameModeSelector } from "../../components/gameModeSettings/GameModeSelector";
import LobbyChatMenu from "./LobbyChatMenu";
import { useLobbyState } from "../../components/useHooks";
import { Button } from "../../components/Button";
import { ModifiersSelector } from "../../components/gameModeSettings/ModifiersSelector";
import LobbyNamePane from "./LobbyNamePane";
import { UnsafeString } from "../../game/gameState.d";
import { encodeString } from "../../components/ChatMessage";
import ListMap from "../../ListMap";
import RandomSeedSelector from "../../components/gameModeSettings/RandomSeedSelector";
import { TabbedContent, TabDefinition } from "../../components/TabbedContent";

export default function LobbyMenu(): ReactElement {
    const isSpectator = useLobbyState(
        lobbyState => lobbyState.players.get(lobbyState.myId!)?.clientType.type === "spectator",
        ["playersHost", "lobbyClients"]
    )!;
    const isHost = useLobbyState(
        lobbyState => {
            const myClient = lobbyState.players.get(lobbyState.myId!);
            if (myClient === null) return true;
            return myClient.ready === "host";
        },
        ["playersHost", "lobbyClients", "yourId"]
    )!;
    const mobile = useContext(MobileContext)!;

    const [advancedView, setAdvancedView] = useState<boolean>(!mobile);

    useEffect(() => {
        setAdvancedView(!mobile)
    }, [mobile, isHost]);

    useEffect(() => {
        const onBeforeUnload = (e: BeforeUnloadEvent) => {
            if (!DEV_ENV) e.preventDefault()
        };

        window.addEventListener("beforeunload", onBeforeUnload);
        return () => window.removeEventListener("beforeunload", onBeforeUnload);
    }, []);

    return <div className="lm">
        <div>
            <LobbyMenuHeader isHost={isHost} advancedView={advancedView} setAdvancedView={setAdvancedView}/>
            {advancedView 
                ? <main>
                    <div>
                        <LobbyNamePane />
                        <LobbyPlayerList />
                        <LobbyChatMenu spectator={isSpectator}/>
                    </div>
                    <div>
                        <LobbyMenuSettings isHost={isHost}/>
                    </div>
                </main>
                : <main>
                    <div>
                        <LobbyNamePane />
                        <LobbyPlayerList />
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
    const roleList = useLobbyState(
        lobbyState => lobbyState.roleList,
        ["roleList", "roleOutline"]
    )!;
    const enabledRoles = useLobbyState(
        lobbyState => lobbyState.enabledRoles,
        ["enabledRoles"]
    )!;
    const phaseTimes = useLobbyState(
        lobbyState => lobbyState.phaseTimes,
        ["phaseTimes"]
    )!;
    const modifierSettings = useLobbyState(
        lobbyState => lobbyState.modifierSettings.list,
        ["modifierSettings"]
    )!;

    const mobile = useContext(MobileContext)!;
    const { setContent: setAnchorContent } = useContext(AnchorControllerContext)!;

    useEffect(() => {
        const listener: StateListener = (type) => {
            if(type === "rejectJoin"){
                // Kicked, probably
                setAnchorContent(<LoadingScreen type="disconnect"/>);
                GAME_MANAGER.setDisconnectedState();
                setAnchorContent(<StartMenu />);
            }
        }

        GAME_MANAGER.addStateListener(listener);
        return ()=>{GAME_MANAGER.removeStateListener(listener);}
    }, [setAnchorContent]);

    const sendRoleList = (newRoleList: RoleList) => {
        const combinedRoleList = structuredClone(roleList);
        newRoleList.forEach((role, index) => {
            combinedRoleList[index] = role
        })
        GAME_MANAGER.sendSetRoleListPacket(combinedRoleList);
    };

    const context = useMemo(() => {
        return {roleList, enabledRoles, phaseTimes, modifierSettings};
    }, [enabledRoles, phaseTimes, roleList, modifierSettings]);

    const tabs: TabDefinition<"phaseTimes" | "modifiers" | "outlineList" | "enabledRoles">[] = useMemo(() => [
        {
            id: "phaseTimes",
            label: translate("menu.lobby.timeSettings"),
            content: <PhaseTimesSelector 
                disabled={!props.isHost}
                onChange={pts => GAME_MANAGER.sendSetPhaseTimesPacket(pts)}
            />
        },
        {
            id: "modifiers",
            label: translate("modifiers"),
            content: <ModifiersSelector
                disabled={!props.isHost}
                setModifiers={modifiers => GAME_MANAGER.sendModifierSettingsPacket(new ListMap(modifiers))}
            />
        },
        {
            id: "outlineList",
            label: translate("menu.lobby.roleList"),
            content: <OutlineListSelector
                disabled={!props.isHost}
                onChangeRolePicker={(value, index) => GAME_MANAGER.sendSetRoleOutlinePacket(index, value)}
                onAddNewOutline={undefined}
                onRemoveOutline={undefined}
                setRoleList={sendRoleList}
            />
        },
        {
            id: "enabledRoles",
            label: translate("menu.lobby.enabledRoles"),
            content: <EnabledRoleSelector
                onEnableRoles={roles => GAME_MANAGER.sendEnabledRolesPacket([...enabledRoles, ...roles])}
                onDisableRoles={roles => GAME_MANAGER.sendEnabledRolesPacket(enabledRoles.filter(role => !roles.includes(role)))}
                onIncludeAll={() => GAME_MANAGER.sendEnabledRolesPacket(getAllRoles())}
                disabled={!props.isHost}
            />
        }
    ], [props.isHost, enabledRoles, sendRoleList]);

    return <GameModeContext.Provider value={context}>
        <div className="lobby-settings-container">
            {mobile && <h1>{translate("menu.lobby.settings")}</h1>}
            {props.isHost === true && <GameModeSelector 
                canModifySavedGameModes={false}
                loadGameMode={gameMode => {
                    GAME_MANAGER.sendSetPhaseTimesPacket(gameMode.phaseTimes);
                    GAME_MANAGER.sendEnabledRolesPacket(gameMode.enabledRoles);
                    GAME_MANAGER.sendSetRoleListPacket(gameMode.roleList);
                    GAME_MANAGER.sendModifierSettingsPacket(new ListMap(gameMode.modifierSettings));
                }}
            />}
            <RandomSeedSelector
                disabled={!props.isHost}
                onChange={randomSeed => GAME_MANAGER.sendSetRandomSeedPacket(randomSeed)}
            />
            <TabbedContent
                tabs={tabs}
                defaultTab="phaseTimes"
            />
        </div>
    </GameModeContext.Provider>
}

// There's probably a better way to do this that doesn't need the mobile check.
function LobbyMenuHeader(props: Readonly<{
    isHost: boolean,
    advancedView: boolean,
    setAdvancedView: (advancedView: boolean) => void
}>): JSX.Element {
    const [lobbyName, setLobbyName] = useState<UnsafeString>(GAME_MANAGER.state.stateType === "lobby" ? GAME_MANAGER.state.lobbyName : "Mafia Lobby");
    const mobile = useContext(MobileContext)!;
    const { setContent: setAnchorContent } = useContext(AnchorControllerContext)!;

    useEffect(() => {
        const listener: StateListener = (type) => {
            if (type === "lobbyName" && GAME_MANAGER.state.stateType === "lobby") {
                setLobbyName(GAME_MANAGER.state.lobbyName);
            }
        };

        if(GAME_MANAGER.state.stateType === "lobby")
            setLobbyName(GAME_MANAGER.state.lobbyName);

        GAME_MANAGER.addStateListener(listener)
        return ()=>{GAME_MANAGER.removeStateListener(listener);}
    }, [setLobbyName]);

    return <header>
        <div>
            <Button disabled={!props.isHost} className="start" onClick={async ()=>{
                setAnchorContent(<LoadingScreen type="default"/>);
                if (!await GAME_MANAGER.sendStartGamePacket()) {
                    setAnchorContent(<LobbyMenu/>)
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
                value={lobbyName as string}
                onInput={e => {
                    setLobbyName((e.target as HTMLInputElement).value);
                }}
                onKeyUp={(e)=>{
                    if(e.key !== 'Enter') return;
                    
                    const newLobbyName = (e.target as HTMLInputElement).value;
                    setLobbyName(newLobbyName);
                    GAME_MANAGER.sendSetLobbyNamePacket(newLobbyName);
                    
                }}
                onBlur={e => {
                    const newLobbyName = (e.target as HTMLInputElement).value;
                    setLobbyName(newLobbyName);
                    GAME_MANAGER.sendSetLobbyNamePacket(newLobbyName);
                }}
            /> : 
            <h3>{encodeString(lobbyName)}</h3>
        }
        
    </header>
}

