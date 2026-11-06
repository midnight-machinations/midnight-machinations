import { createContext, type ReactElement, useCallback, useContext, useEffect, useState } from "react";
import GAME_MANAGER, { DEV_ENV, type Theme } from "../..";
import ChatMenu from "./gameScreenContent/ChatMenu";
import GraveyardMenu from "./gameScreenContent/GraveyardMenu";
import PlayerListMenu from "./gameScreenContent/PlayerListMenu";
import WikiMenu from "./gameScreenContent/WikiMenu";
import WillMenu from "./gameScreenContent/WillMenu";
import HeaderMenu, { MenuButtons } from "./HeaderMenu";
import "../../index.css";
import "./gameScreen.css";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { Button } from "../../components/Button";
import Icon from "../../components/Icon";
import StyledText from "../../components/StyledText";
import { useGameState } from "../../components/useHooks";
import translate from "../../game/lang";
import { loadSettingsParsed } from "../../game/localStorage";
import { MobileContext } from "../Anchor";
import AbilityMenu from "./gameScreenContent/AbilityMenu/AbilityMenu";
import { getNextSwipeMenu, useMenuSwipe } from "./useMenuSwipe";

export enum ContentMenu {
    ChatMenu = "ChatMenu",
    RoleSpecificMenu = "RoleSpecificMenu",
    WillMenu = "WillMenu",
    PlayerListMenu = "PlayerListMenu",
    GraveyardMenu = "GraveyardMenu",
    WikiMenu = "WikiMenu",
}

export const MENU_ELEMENTS = {
    [ContentMenu.ChatMenu]: ChatMenu,
    [ContentMenu.PlayerListMenu]: PlayerListMenu,
    [ContentMenu.RoleSpecificMenu]: AbilityMenu,
    [ContentMenu.WillMenu]: WillMenu,
    [ContentMenu.GraveyardMenu]: GraveyardMenu,
    [ContentMenu.WikiMenu]: WikiMenu,
};

export const MENU_THEMES: Record<ContentMenu, Theme | null> = {
    [ContentMenu.ChatMenu]: "chat-menu-colors",
    [ContentMenu.PlayerListMenu]: "player-list-menu-colors",
    [ContentMenu.RoleSpecificMenu]: "role-specific-colors",
    [ContentMenu.WillMenu]: "will-menu-colors",
    [ContentMenu.GraveyardMenu]: "graveyard-menu-colors",
    [ContentMenu.WikiMenu]: "wiki-menu-colors",
};

export const MENU_TRANSLATION_KEYS: Record<ContentMenu, string> = {
    [ContentMenu.ChatMenu]: "menu.chat",
    [ContentMenu.PlayerListMenu]: "menu.playerList",
    [ContentMenu.RoleSpecificMenu]: "menu.ability",
    [ContentMenu.WillMenu]: "menu.will",
    [ContentMenu.GraveyardMenu]: "menu.gameMode",
    [ContentMenu.WikiMenu]: "menu.wiki",
};

export interface MenuController {
    closeOrOpenMenu(menu: ContentMenu): void;
    closeMenu(menu: ContentMenu): void;
    openMenu(menu: ContentMenu, callback?: () => void): void;
    menusOpen(): ContentMenu[];
    menuOpen(menu: ContentMenu): boolean;
    canOpen(menu: ContentMenu): boolean;
    menus(): ContentMenu[];
    maxMenus: number;
}

export function useMenuController<C extends Partial<Record<ContentMenu, boolean>>>(
    maxContent: number,
    initial: C,
    getMenuController: () => MenuController,
    setMenuController: (menuController: MenuController | undefined) => void,
): MenuController {
    const [contentMenus, setContentMenus] = useState<C>(initial);

    const [callbacks, setCallbacks] = useState<(() => void)[]>([]);

    useEffect(() => {
        for (const callback of callbacks) {
            callback();
        }
        if (callbacks.length !== 0) {
            setCallbacks([]);
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [contentMenus]);

    const initializeMenuController = useCallback(() => {
        function setContentMenu(menu: ContentMenu, open: boolean) {
            setContentMenus((previous) => {
                if (previous[menu] === undefined) return previous;
                const newMenus = { ...previous };
                newMenus[menu] = open;
                const menusOpen = Object.entries(previous)
                    .filter(([other, isOpen]) => isOpen && other !== menu)
                    .map(([other]) => other as ContentMenu);
                if (open && menusOpen.length >= maxContent) {
                    newMenus[menusOpen[menusOpen.length - 1]] = false;
                }
                return newMenus;
            });
        }

        setMenuController({
            closeMenu(menu) {
                setContentMenu(menu, false);

                if (GAME_MANAGER.state.stateType === "game" && menu === ContentMenu.ChatMenu) {
                    GAME_MANAGER.state.missedChatMessages = false;
                }
                GAME_MANAGER.invokeStateListeners("closeGameMenu");
            },
            closeOrOpenMenu(menu) {
                if (getMenuController().menusOpen().includes(menu)) {
                    getMenuController().closeMenu(menu);
                } else {
                    getMenuController().openMenu(menu, () => {});
                }
            },
            openMenu(menu, callback) {
                if (
                    GAME_MANAGER.state.stateType === "game" &&
                    GAME_MANAGER.state.clientState.type === "spectator" &&
                    (menu === ContentMenu.WillMenu || menu === ContentMenu.RoleSpecificMenu)
                ) {
                    return;
                }
                setContentMenu(menu, true);

                if (GAME_MANAGER.state.stateType === "game" && menu === ContentMenu.ChatMenu) {
                    GAME_MANAGER.state.missedChatMessages = false;
                }

                if (callback) {
                    setCallbacks((callbacks) => callbacks.concat(callback));
                }

                GAME_MANAGER.invokeStateListeners("openGameMenu");
            },
            menusOpen(): ContentMenu[] {
                return Object.entries(contentMenus)
                    .filter(([_, open]) => open)
                    .map(([menu, _]) => menu) as ContentMenu[];
            },
            menuOpen(menu): boolean {
                return this.menusOpen().includes(menu);
            },
            canOpen(menu): boolean {
                if (
                    GAME_MANAGER.state.stateType === "game" &&
                    GAME_MANAGER.state.clientState.type === "spectator" &&
                    (menu === ContentMenu.WillMenu || menu === ContentMenu.RoleSpecificMenu)
                ) {
                    return false;
                }
                return contentMenus[menu] !== undefined;
            },
            menus(): ContentMenu[] {
                return Object.keys(contentMenus).filter(
                    (menu) => contentMenus[menu as ContentMenu] !== undefined,
                ) as ContentMenu[];
            },
            maxMenus: maxContent,
        });
    }, [contentMenus, getMenuController, maxContent, setMenuController]);

    // Initialize on component load so MenuButtons component doesn't freak out
    initializeMenuController();
    useEffect(() => {
        initializeMenuController();
        return () => setMenuController(undefined);
    }, [initializeMenuController, setMenuController]);

    return getMenuController();
}

const MENU_CONTROLLER_HOLDER: { controller: MenuController | undefined } = {
    controller: undefined,
};

const MenuControllerContext = createContext<MenuController | undefined>(undefined);

export { MenuControllerContext };

export default function GameScreen(): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    const menuController = useMenuController(
        maxMenus,
        Object.fromEntries(menuOrder),
        () => MENU_CONTROLLER_HOLDER.controller!,
        (menuController) => (MENU_CONTROLLER_HOLDER.controller = menuController),
    );

    const chatMenuNotification = useGameState(
        (game) => game.missedChatMessages && !menuController.menuOpen(ContentMenu.ChatMenu),
        ["addChatMessages", "openGameMenu", "closeGameMenu"],
    )!;

    useEffect(() => {
        const onBeforeUnload = (e: BeforeUnloadEvent) => {
            if (!DEV_ENV) e.preventDefault();
        };

        window.addEventListener("beforeunload", onBeforeUnload);
        return () => window.removeEventListener("beforeunload", onBeforeUnload);
    }, []);

    return (
        <MenuControllerContext.Provider value={menuController}>
            <div className="game-screen">
                <div className="header">
                    <HeaderMenu chatMenuNotification={chatMenuNotification} />
                </div>
                <GameScreenMenus />
                {mobile && <MenuButtons chatMenuNotification={chatMenuNotification} />}
            </div>
        </MenuControllerContext.Provider>
    );
}

export function GameScreenMenus(): ReactElement {
    const menuController = useContext(MenuControllerContext)!;
    const minSize = 10; // Percentage
    const mobile = useContext(MobileContext)!;
    const swipeHandlers = useMenuSwipe(mobile, (right) => {
        const open = menuController.menusOpen().filter((menu) => menuController.canOpen(menu));
        const next = getNextSwipeMenu(
            menuController.menus().filter((menu) => menuController.canOpen(menu)),
            open,
            right,
        );
        if (next === undefined) return;
        menuController.closeMenu(open[open.length - 1]);
        menuController.openMenu(next);
    });

    // These don't add up to 100, but the panel group will fix it
    const defaultSizes = {
        [ContentMenu.ChatMenu]: 35,
        [ContentMenu.RoleSpecificMenu]: 15,
        [ContentMenu.WillMenu]: 15,
        [ContentMenu.PlayerListMenu]: 25,
        [ContentMenu.GraveyardMenu]: 10,
        [ContentMenu.WikiMenu]: 15,
    };

    // Filter out menus that shouldn't be shown
    const visibleMenus = menuController.menusOpen().filter((menu) => {
        if (
            GAME_MANAGER.state.stateType === "game" &&
            GAME_MANAGER.state.clientState.type === "spectator" &&
            (menu === ContentMenu.WillMenu || menu === ContentMenu.RoleSpecificMenu)
        ) {
            return false;
        }
        return true;
    });

    return (
        <div className="game-menu-swipe-container" {...swipeHandlers}>
            <PanelGroup direction="horizontal" className="content">
                {visibleMenus.flatMap((menu, index) => {
                    const MenuElement = MENU_ELEMENTS[menu];

                    const out = [
                        <Panel
                            id={menu}
                            order={index}
                            className="panel"
                            minSize={minSize}
                            defaultSize={mobile === false ? defaultSizes[menu] : undefined}
                            key={`${menu}.panel`}
                        >
                            <MenuElement />
                        </Panel>,
                    ];

                    if (!mobile && visibleMenus.length > index + 1) {
                        const nextMenu = visibleMenus[index + 1];
                        out.push(<PanelResizeHandle key={`${menu}.and.${nextMenu}.handle`} className="panel-handle" />);
                    }
                    return out;
                })}
                {visibleMenus.length === 0 && (
                    <Panel>
                        <div className="no-content">
                            <StyledText noLinks={true}>{translate("menu.gameScreen.noContent")}</StyledText>
                        </div>
                    </Panel>
                )}
            </PanelGroup>
        </div>
    );
}

export function ContentTab(
    props: Readonly<{
        close: ContentMenu | false;
        children: string;
    }>,
): ReactElement {
    const menuController = useContext(MenuControllerContext)!;
    const spectator = useGameState((gameState) => gameState.clientState.type === "spectator", ["gamePlayers"])!;
    const mobile = useContext(MobileContext)!;

    return (
        <div className="content-tab">
            <StyledText>{props.children}</StyledText>

            {props.close && (!spectator || mobile) && (
                <Button className="close flush" onClick={() => menuController.closeMenu(props.close as ContentMenu)}>
                    <Icon>close</Icon>
                </Button>
            )}
        </div>
    );
}
