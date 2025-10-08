import React, { JSXElementConstructor, ReactElement, useContext, useEffect, useRef } from 'react';
import "./globalMenu.css";
import translate from '../game/lang';
import GAME_MANAGER from '..';
import { AnchorControllerContext } from './Anchor';
import StartMenu from './main/StartMenu';
import LoadingScreen from './LoadingScreen';
import GameModesEditor from '../components/gameModeSettings/GameModesEditor';
import { CopyButton } from '../components/ClipboardButtons';
import WikiCoverCard from '../components/WikiCoverCard';
import Icon from '../components/Icon';
import SettingsMenu from './Settings';
import { useLobbyOrGameState } from '../components/useHooks';
import { Button } from '../components/Button';
import HostMenu from './HostMenu';
import { useAuth0 } from '@auth0/auth0-react';
import Profile from './Profile';
import { encodeString } from '../components/ChatMessage';

export default function GlobalMenu(): ReactElement {
    const lobbyName = useLobbyOrGameState(
        state => state.lobbyName,
        ["lobbyName"]
    )!;
    const host = useLobbyOrGameState(
        state => {
            if (state.stateType === "game") {
                return state.host !== null
            } else {
                return state.players.get(state.myId!)?.ready === "host"
            }
        },
        ["lobbyClients", "playersHost", "gamePlayers"]
    )!;
    const stateType = useLobbyOrGameState(
        state => state.stateType,
        ["acceptJoin", "rejectJoin", "rejectStart", "gameInitializationComplete", "startGame"]
    )!;
    const ref = useRef<HTMLDivElement>(null);
    const anchorController = useContext(AnchorControllerContext)!;
    const { isAuthenticated, loginWithRedirect } = useAuth0();

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (!ref.current?.contains(event.target as Node)) {
                anchorController.closeGlobalMenu();
            }
        };

        setTimeout(() => {
            document.addEventListener("click", handleClickOutside);
        })
        return () => document.removeEventListener("click", handleClickOutside);
    }, [anchorController]);
    
    async function quitToMainMenu() {
        if (stateType === "game") {
            GAME_MANAGER.leaveGame();
        }
        anchorController.closeGlobalMenu();
        anchorController.clearCoverCard();
        anchorController.setContent(<LoadingScreen type="disconnect"/>)
        window.history.replaceState({}, '', '/')
        await GAME_MANAGER.setDisconnectedState();
        anchorController.setContent(<StartMenu/>)
    }
    function goToRolelistEditor() {
        anchorController.setCoverCard(<GameModesEditor/>);
        anchorController.closeGlobalMenu();
    }
    const quitButtonBlacklist: (string | JSXElementConstructor<any>)[] = [StartMenu, LoadingScreen];

    return (
        <div className="chat-menu-colors global-menu slide-in" ref={ref}>
            {(stateType === "lobby" || stateType === "game") && 
                <section className="standout">
                    <h2>{encodeString(lobbyName)}</h2>
                    <RoomLinkButton/>
                    {(stateType === "game" && host) && <>
                        <Button onClick={()=>GAME_MANAGER.sendBackToLobbyPacket()}>
                            {translate("backToLobby")}
                        </Button>
                        <Button onClick={()=>anchorController.setCoverCard(<HostMenu />)}>
                            {translate("menu.hostSettings.title")}
                        </Button>
                    </>}
                </section>
            }
            <section>
                { quitButtonBlacklist.includes(anchorController.contentType) ||
                    <button onClick={() => quitToMainMenu()}><Icon>not_interested</Icon> {translate("menu.globalMenu.quitToMenu")}</button>
                }
                {isAuthenticated 
                    ? <button onClick={() => anchorController.setCoverCard(<Profile />)}><Icon>person</Icon>{translate("menu.profile.title")}</button>
                    : <button onClick={() => loginWithRedirect()}><Icon>login</Icon>{translate("login")}</button>
                }
                <button onClick={() => {
                    anchorController.setCoverCard(<SettingsMenu />)
                    anchorController.closeGlobalMenu();
                }}><Icon>settings</Icon> {translate("menu.globalMenu.settings")}</button>
                <button onClick={() => goToRolelistEditor()}><Icon>edit</Icon> {translate("menu.globalMenu.gameSettingsEditor")}</button>
                <button onClick={() => {
                    anchorController.setCoverCard(<WikiCoverCard />);
                    anchorController.closeGlobalMenu();
                }}><Icon>menu_book</Icon> {translate("menu.wiki.title")}</button>
            </section>
        </div>
    );
}

export function RoomLinkButton(): JSX.Element {
    const code = useLobbyOrGameState(
        state => {
            const code = new URL(window.location.href);
            code.pathname = "/connect"
            code.searchParams.set("code", state.roomCode.toString(18))
            return code;
        }, ["acceptJoin", "backToLobby"]
    )!;
    
    return <CopyButton text={code.toString()}>
        <Icon>link</Icon> {translate("menu.play.field.roomCode")}
    </CopyButton>
}