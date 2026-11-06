import { type JSXElementConstructor, type ReactElement, useContext, useEffect, useRef } from "react";
import "./globalMenu.css";
import GAME_MANAGER from "..";
import { Button } from "../components/Button";
import { encodeString } from "../components/ChatMessage";
import { CopyButton } from "../components/ClipboardButtons";
import GameModesEditor from "../components/gameModeSettings/GameModesEditor";
import Icon from "../components/Icon";
import { useLobbyOrGameState } from "../components/useHooks";
import WikiCoverCard from "../components/WikiCoverCard";
import translate from "../game/lang";
import { AnchorControllerContext } from "./Anchor";
import HostMenu from "./HostMenu";
import LoadingScreen from "./LoadingScreen";
import StartMenu from "./main/StartMenu";
import SettingsMenu from "./Settings";

export default function GlobalMenu(): ReactElement {
    const lobbyName = useLobbyOrGameState((state) => state.lobbyName, ["lobbyName"])!;
    const host = useLobbyOrGameState(
        (state) => {
            if (state.stateType === "game") {
                return state.host !== null;
            } else {
                return state.players.get(state.myId!)?.ready === "host";
            }
        },
        ["lobbyClients", "playersHost", "gamePlayers"],
    )!;
    const stateType = useLobbyOrGameState(
        (state) => state.stateType,
        ["acceptJoin", "rejectJoin", "rejectStart", "gameInitializationComplete", "startGame"],
    )!;
    const ref = useRef<HTMLDivElement>(null);
    const anchorController = useContext(AnchorControllerContext)!;

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (!ref.current?.contains(event.target as Node)) {
                anchorController.closeGlobalMenu();
            }
        };

        setTimeout(() => {
            document.addEventListener("click", handleClickOutside);
        });
        return () => document.removeEventListener("click", handleClickOutside);
    }, [anchorController]);

    async function quitToMainMenu() {
        if (stateType === "game") {
            GAME_MANAGER.leaveGame();
        }
        anchorController.closeGlobalMenu();
        anchorController.clearCoverCard();
        anchorController.setContent(<LoadingScreen type="disconnect" />);
        window.history.replaceState({}, "", "/");
        await GAME_MANAGER.setDisconnectedState();
        anchorController.setContent(<StartMenu />);
    }
    function goToRolelistEditor() {
        anchorController.setCoverCard(<GameModesEditor />);
        anchorController.closeGlobalMenu();
    }
    const quitButtonBlacklist: (string | JSXElementConstructor<never>)[] = [StartMenu, LoadingScreen];

    return (
        <div className="global-menu slide-in" ref={ref}>
            {(stateType === "lobby" || stateType === "game") && (
                <section className="standout">
                    <h2>{encodeString(lobbyName)}</h2>
                    <RoomLinkButton verbose />
                    {stateType === "game" && host && (
                        <>
                            <Button onClick={() => GAME_MANAGER.sendBackToLobbyPacket()}>
                                {translate("backToLobby")}
                            </Button>
                            <Button onClick={() => anchorController.setCoverCard(<HostMenu />)}>
                                {translate("menu.hostSettings.title")}
                            </Button>
                        </>
                    )}
                    <Button onClick={() => quitToMainMenu()}>
                        <Icon>not_interested</Icon> {translate("menu.globalMenu.abandon")}
                    </Button>
                </section>
            )}
            <section>
                {stateType === "lobby" ||
                    stateType === "game" ||
                    quitButtonBlacklist.includes(anchorController.contentType) || (
                        <Button onClick={() => quitToMainMenu()}>
                            <Icon>not_interested</Icon> {translate("menu.globalMenu.quitToMenu")}
                        </Button>
                    )}
                <Button
                    onClick={() => {
                        anchorController.setCoverCard(<SettingsMenu />);
                        anchorController.closeGlobalMenu();
                    }}
                >
                    <Icon>settings</Icon> {translate("menu.globalMenu.settings")}
                </Button>
                <Button onClick={() => goToRolelistEditor()}>
                    <Icon>edit</Icon> {translate("menu.globalMenu.gameSettingsEditor")}
                </Button>
                <Button
                    onClick={() => {
                        anchorController.setCoverCard(<WikiCoverCard />);
                        anchorController.closeGlobalMenu();
                    }}
                >
                    <Icon>menu_book</Icon> {translate("menu.wiki.title")}
                </Button>
            </section>
        </div>
    );
}

export function RoomLinkButton(props: { verbose?: true }): JSX.Element {
    const code = useLobbyOrGameState(
        (state) => {
            const code = new URL(window.location.href);
            code.pathname = "/connect";
            code.searchParams.set("code", state.roomCode.toString(18));
            return code;
        },
        ["acceptJoin", "backToLobby"],
    )!;

    return (
        <CopyButton className={props.verbose ? "" : "flush"} text={code.toString()}>
            <Icon>link_2</Icon> {props.verbose && translate("menu.lobby.button.copyLink")}
        </CopyButton>
    );
}
