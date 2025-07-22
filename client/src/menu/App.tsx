import React, {
    ReactElement, useRef, useContext,
    useEffect, useCallback
} from "react";
import "../index.css";
import "./app.css";
import translate, { switchLanguage } from "../game/lang";
import GlobalMenu from "./GlobalMenu";
import { deleteReconnectData, loadSettingsParsed, saveReconnectData } from "../game/localStorage";
import { Theme } from "..";
import Icon from "../components/Icon";
import { Button } from "../components/Button";
import { ChatMessage } from "../components/ChatMessage";
import AudioController from "./AudioController";
import { computeKeywordData } from "../components/StyledText";
import AppContextProvider, { AppContext, AppContextType } from "./AppContext";
import MobileContextProvider from "./MobileContext";
import WebsocketContextProvider, { WebsocketContext, WebSocketContextType } from "./WebsocketContext";
import { ToClientPacket } from "../packet";
import StateContextProvider from "../stateContext/StateContextProvider";

export default function App(props: Readonly<{
    onMount: (appContext: AppContextType, websocketContext: WebSocketContextType) => void,
}>): ReactElement {
    return <MobileContextProvider>
        <WebsocketContextProvider>
            <AppContextProvider>
                <StateContextProvider>
                    <AppInner {...props}/>
                </StateContextProvider>
            </AppContextProvider>
        </WebsocketContextProvider>
    </MobileContextProvider>
}

function AppInner(props: Readonly<{
    onMount: (appContext: AppContextType, websocketContext: WebSocketContextType) => void,
}>): ReactElement {
    const appContext = useContext(AppContext)!;
    const websocketContext = useContext(WebsocketContext)!;

    // Load settings
    useEffect(() => {
        const settings = loadSettingsParsed();

        AudioController.setVolume(settings.volume);
        appContext.setFontSize(settings.fontSize);
        appContext.setAccessibilityFontEnabled(settings.accessibilityFont);
        switchLanguage(settings.language);
        computeKeywordData();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    useEffect(() => {
        props.onMount(appContext, websocketContext);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [props])

    // useEffect(() => {
    //     if (websocketContext.lastMessageRecieved) {
    //         appMessageListener(websocketContext.lastMessageRecieved, appContext, websocketContext)
    //     }
    //     // eslint-disable-next-line react-hooks/exhaustive-deps
    // }, [websocketContext.lastMessageRecieved]);
    
    useEffect(() => {
        websocketContext.awaitCloseOrError().then(type => {
            if (type === "close") {
                appContext?.pushErrorCard({
                    title: translate("notification.connectionFailed"), 
                    body: ""
                });
                appContext?.setContent({type:"main"});
            } else if (type === "error") {
                appContext?.pushErrorCard({
                    title: translate("notification.connectionFailed"), 
                    body: translate("notification.serverNotFound")
                });
            }
        })
        // Don't need to close the listener, won't mess anything up except take some memory
    }, [appContext, websocketContext])

    return <div className="anchor">
        <Button className="global-menu-button" 
            onClick={() => {
                if (!appContext.globalMenuOpen) {
                    appContext.openGlobalMenu()
                }else{
                    appContext.openGlobalMenu()
                }
            }}
        >
            <Icon>menu</Icon>
        </Button>
        {appContext.globalMenuOpen && <GlobalMenu />}
        {appContext.content}
        {appContext.coverCard && <CoverCard
            theme={appContext.coverCardTheme}
        >{appContext.coverCard}</CoverCard>}
        {appContext.errorCard}
    </div>
}

function CoverCard(props: Readonly<{
    children: React.ReactNode,
    theme: Theme | null
}>): ReactElement {
    const ref = useRef<HTMLDivElement>(null);
    const appController = useContext(AppContext)!;

    const escFunction = useCallback((event: KeyboardEvent) =>{
        if(event.key === "Escape") {
            appController.clearCoverCard();
        }
    }, [appController]);
    useEffect(() => {
        document.addEventListener("keydown", escFunction, false);
        return () => {
            document.removeEventListener("keydown", escFunction, false);
        };
    }, [escFunction]);
    return <div 
        className={`anchor-cover-card-background-cover ${props.theme ?? ""}`} 
        onClick={e => {
            if (e.target === ref.current) appController.clearCoverCard()
        }}
        ref={ref}
    >
        <div className="anchor-cover-card">
            <Button className="close-button" onClick={appController.clearCoverCard}>
                <Icon>close</Icon>
            </Button>
            <div className="anchor-cover-card-content">
                {props.children}
            </div>
        </div>
    </div>
}

export type ErrorData = {
    title: string,
    body: string
}

export function ErrorCard(props: Readonly<{
    error: ErrorData,
    onClose: () => void
}>) {
    return <div className="error-card" onClick={() => props.onClose()}>
        <header>
            {props.error.title}
            <button className="close">✕</button>
        </header>
        <div>{props.error.body}</div>
    </div>
}


export type AudioFile = 
    "church_bell.mp3" | 
    "alarm.mp3" | 
    "vine_boom.mp3" | 
    "sniper_shot.mp3" | 
    "normal_message.mp3" | 
    "whisper_broadcast.mp3" | 
    "start_game.mp3";

export type AudioFilePath = `audio/${AudioFile}`;

export function chatMessageToAudio(msg: ChatMessage): AudioFilePath | null {
    let file: AudioFile|null = null;

    switch(msg.variant.type){
        case "normal":
        case "voted":
            file = "normal_message.mp3";
            break;
        case "broadcastWhisper":
            file = "whisper_broadcast.mp3";
            break;
        case "playerDied": 
            file = "church_bell.mp3";
            break;
        case "deputyKilled": 
            file = "sniper_shot.mp3";
            break;
    }

    if(file){
        return `audio/${file}`;
    }else{
        return null
    }
}

export function sendDefaultName(websocketContext: WebSocketContextType) {
    const defaultName = loadSettingsParsed().defaultName;
    if(defaultName !== null && defaultName !== undefined && defaultName !== ""){
        websocketContext.sendSetNamePacket(defaultName)
    }
}

function appMessageListener(packet: ToClientPacket, appContext: AppContextType, websocketContext: WebSocketContextType){
    console.log("useeffect saw:"+packet.type);

    
    switch(packet.type) {
        case "pong":
            websocketContext.sendPacket({
                type: "ping"
            });
        break;
        case "rateLimitExceeded":
            appContext.pushErrorCard({ title: translate("notification.rateLimitExceeded"), body: "" });
        break;
        case "forcedOutsideLobby":
            appContext.setContent({type:"gameBrowser"});
        break;
        case "forcedDisconnect":
            appContext.setContent({type:"main"});
        break
        case "acceptJoin":
            if(packet.inGame && packet.spectator){
                //waiting for gameInitialization, will get set to gamescreen when X packet recieved?
                appContext.setContent({type:"loading"});
            }else if(packet.inGame && !packet.spectator){
                //waiting for gameInitialization, will get set to gamescreen when X packet recieved?
                appContext.setContent({type:"loading"});
            }

            saveReconnectData(packet.roomCode, packet.playerId);
            sendDefaultName(websocketContext);
            appContext.clearCoverCard();
        break;
        case "rejectJoin":
            switch(packet.reason) {
                case "roomDoesntExist":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomDoesntExist") });
                    // If the room doesn't exist, don't suggest the user to reconnect to it.
                    deleteReconnectData();
                    appContext.clearCoverCard();
                break;
                case "gameAlreadyStarted":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.gameAlreadyStarted") });
                break;
                case "roomFull":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomFull") });
                break;
                case "serverBusy":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.serverBusy") });
                break;
                case "playerTaken":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerTaken") });
                break;
                case "playerDoesntExist":
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerDoesntExist") });
                break;
                default:
                    appContext.pushErrorCard({ title: translate("notification.rejectJoin"), body: `${packet.type} message response not implemented: ${packet.reason}` });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
            deleteReconnectData();
            
        break;
        // default:
        //     console.error(`incoming message response not implemented: ${(packet as any)?.type}`);
        //     console.error(packet);
        // break;
    }
}