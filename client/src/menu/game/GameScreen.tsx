import React, { ReactElement, useContext, useEffect } from "react";
import HeaderMenu, { MenuButtons } from "./HeaderMenu";
import GraveyardMenu from "./gameScreenContent/GraveyardMenu";
import ChatMenu from "./gameScreenContent/ChatMenu";
import PlayerListMenu from "./gameScreenContent/PlayerListMenu";
import WillMenu from "./gameScreenContent/WillMenu";
import GAME_MANAGER, { DEV_ENV, Theme } from "../..";
import WikiMenu from "./gameScreenContent/WikiMenu";
import "../../index.css";
import "./gameScreen.css";
import AbilityMenu from "./gameScreenContent/AbilityMenu/AbilityMenu";
import { MobileContext } from "../Anchor";
import StyledText from "../../components/StyledText";
import Icon from "../../components/Icon";
import { Button } from "../../components/Button";
import translate from "../../game/lang";
import { useGameState } from "../../components/useHooks";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { loadSettingsParsed } from "../../game/localStorage";
import { ContentMenu, MenuControllerContext, useMenuController } from "./menuController";

export const MENU_ELEMENTS = {
    "ChatMenu": ChatMenu,
    "PlayerListMenu": PlayerListMenu,
    "RoleSpecificMenu": AbilityMenu,
    "WillMenu": WillMenu,
    "GraveyardMenu": GraveyardMenu,
    "WikiMenu": WikiMenu
}

export const MENU_THEMES: Record<ContentMenu, Theme | null> = {
    "ChatMenu": "chat-menu-colors",
    "PlayerListMenu": "player-list-menu-colors",
    "RoleSpecificMenu": "role-specific-colors",
    "WillMenu": "will-menu-colors",
    "GraveyardMenu": "graveyard-menu-colors",
    "WikiMenu": "wiki-menu-colors"
}

export const MENU_TRANSLATION_KEYS: Record<ContentMenu, string> = {
    "ChatMenu": "menu.chat",
    "PlayerListMenu": "menu.playerList",
    "RoleSpecificMenu": "menu.ability",
    "WillMenu": "menu.will",
    "GraveyardMenu": "menu.gameMode",
    "WikiMenu": "menu.wiki"
}

const defaultSizes = {
    "ChatMenu": 35,
    "RoleSpecificMenu": 15,
    "WillMenu": 15,
    "PlayerListMenu": 25,
    "GraveyardMenu": 10,
    "WikiMenu": 15,
}


export default function GameScreen(): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    const menuController = useMenuController(
        maxMenus, 
        menuOrder.filter((kvp)=>kvp[1]).map((kvp)=>kvp[0]),
        GAME_MANAGER.state.stateType==="game"&&GAME_MANAGER.state.clientState.type==="spectator"
    );

    const chatMenuNotification = useGameState(
        (game) => game.missedChatMessages && !menuController.menuOpen("ChatMenu"),
        ["addChatMessages", "openGameMenu", "closeGameMenu"]
    )!;
    

    useEffect(() => {
        const onBeforeUnload = (e: BeforeUnloadEvent) => {
            if (!DEV_ENV) e.preventDefault()
        };

        window.addEventListener("beforeunload", onBeforeUnload);
        return () => window.removeEventListener("beforeunload", onBeforeUnload);
    }, [])

    return <MenuControllerContext.Provider value={menuController}>
        <div className="game-screen">
            <div className="header">
                <HeaderMenu chatMenuNotification={chatMenuNotification}/>
            </div>
            <GameScreenMenus/>
            {mobile && <MenuButtons chatMenuNotification={chatMenuNotification}/>}
        </div>
    </MenuControllerContext.Provider>
}

export function GameScreenMenus(): ReactElement {
    const menuController = useContext(MenuControllerContext)!;
    const minSize = 10; // Percentage
    const mobile = useContext(MobileContext)!;


    return <PanelGroup direction="horizontal" className="content">
        {menuController
            .menusOpen()
            .flatMap((menu, index, menusOpen) => {

                if(
                    GAME_MANAGER.state.stateType === "game" &&
                    GAME_MANAGER.state.clientState.type === "spectator" &&
                    (menu === "WillMenu" || menu === "RoleSpecificMenu")
                ){
                    return null;
                }

                const MenuElement = (MENU_ELEMENTS as any)[menu];

                const out = [<Panel
                    className="panel"
                    minSize={minSize}
                    defaultSize={mobile===false?(defaultSizes as any)[menu]:undefined}
                    key={index.toString()+".panel"}
                >
                    <MenuElement />
                </Panel>];

                if(!mobile && menusOpen.length > index + 1){
                    out.push(<PanelResizeHandle key={index.toString()+".handle"} className="panel-handle"/>)
                }
                return out;

            })
        }
        {menuController.menusOpen().length === 0 && <Panel><div className="no-content">
            {translate("menu.gameScreen.noContent")}
        </div></Panel>}
    </PanelGroup>
}

export function ContentTab(props: Readonly<{
    close: ContentMenu | false, 
    children: string 
}>): ReactElement {
    const menuController = useContext(MenuControllerContext)!;
    const spectator = useGameState(
        gameState => gameState.clientState.type === "spectator",
        ["gamePlayers"]
    )!;
    const mobile = useContext(MobileContext)!;

    return <div className="content-tab">
        <div>
            <StyledText>
                {props.children}
            </StyledText>
        </div>

        {props.close && (!spectator || mobile) && <Button className="close"
            onClick={()=>menuController.closeMenu(props.close as ContentMenu)}
        >
            <Icon size="small">close</Icon>
        </Button>}
    </div>
}