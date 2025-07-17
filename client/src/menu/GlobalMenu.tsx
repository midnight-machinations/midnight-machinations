import React, {JSXElementConstructor, ReactElement, useContext, useEffect, useRef } from 'react';
import "./globalMenu.css";
import translate from '../game/lang';
import LoadingScreen from './LoadingScreen';
import GameModesEditor from '../components/gameModeSettings/GameModesEditor';
import { CopyButton } from '../components/ClipboardButtons';
import WikiCoverCard from '../wiki/WikiCoverCard';
import Icon from '../components/Icon';
import SettingsMenu from './Settings';
import { Button } from '../components/Button';
import HostMenu from './HostMenu';
import { AppContext } from './AppContext';
import { WebsocketContext } from './WebsocketContext';
import { deleteReconnectData } from '../game/localStorage';
import { StateContext } from '../stateContext/StateContext';

export default function GlobalMenu(): ReactElement {
    const { sendBackToLobbyPacket } = useContext(WebsocketContext)!;
    const stateCtx = useContext(StateContext)!;

    const lobbyName = stateCtx.lobbyName;
    const host = stateCtx.host;
    const stateType = stateCtx.type;

    const ref = useRef<HTMLDivElement>(null);
    const anchorController = useContext(AppContext)!;
    const websocketContext = useContext(WebsocketContext)!;

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
            websocketContext.sendPacket({ type: "leave" })
            deleteReconnectData();
            websocketContext.close();
        }

        anchorController.closeGlobalMenu();
        anchorController.clearCoverCard();
        anchorController.setContent(<LoadingScreen type="disconnect"/>)
        window.history.replaceState({}, '', '/');
        anchorController.setContent({type:"main"})
    }
    function goToRolelistEditor() {
        anchorController.setCoverCard(<GameModesEditor/>);
        anchorController.closeGlobalMenu();
    }
    const quitButtonBlacklist: (string | JSXElementConstructor<any>)[] = ["main"];

    return (
        <div className="chat-menu-colors global-menu slide-in" ref={ref}>
            {(stateType === "lobby" || stateType === "game") && 
                <section className="standout">
                    <h2>{lobbyName}</h2>
                    <RoomLinkButton/>
                    {(stateType === "game" && host) && <>
                        <Button onClick={()=>sendBackToLobbyPacket()}>
                            {translate("backToLobby")}
                        </Button>
                        <Button onClick={()=>anchorController.setCoverCard(<HostMenu />)}>
                            {translate("menu.hostSettings.title")}
                        </Button>
                    </>}
                </section>
            }
            <section>
                {quitButtonBlacklist.includes(anchorController.contentType.type) ||
                    <Button onClick={() => quitToMainMenu()}><Icon>not_interested</Icon> {translate("menu.globalMenu.quitToMenu")}</Button>
                }
                <Button onClick={() => {
                    anchorController.setCoverCard(<SettingsMenu />)
                    anchorController.closeGlobalMenu();
                }}><Icon>settings</Icon> {translate("menu.globalMenu.settings")}</Button>
                <Button onClick={() => goToRolelistEditor()}><Icon>edit</Icon> {translate("menu.globalMenu.gameSettingsEditor")}</Button>
                <Button onClick={() => {
                    anchorController.setCoverCard(<WikiCoverCard />);
                    anchorController.closeGlobalMenu();
                }}><Icon>menu_book</Icon> {translate("menu.wiki.title")}</Button>
            </section>
        </div>
    );
}

export function RoomLinkButton(): JSX.Element {
    const roomCode = useContext(StateContext)!.roomCode??"";
    const url = new URL(window.location.href);
    url.pathname = "/connect";
    url.searchParams.set("code", roomCode.toString(18))
    
    return <CopyButton text={url.toString()}>
        <Icon>link</Icon> {translate("menu.play.field.roomCode")}
    </CopyButton>
}