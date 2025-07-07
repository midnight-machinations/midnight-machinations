import { createContext, useContext, useEffect, useState } from "react";
import ChatMenu from "./gameScreenContent/ChatMenu";
import PlayerListMenu from "./gameScreenContent/PlayerListMenu";
import AbilityMenu from "./gameScreenContent/AbilityMenu/AbilityMenu";
import WillMenu from "./gameScreenContent/WillMenu";
import GraveyardMenu from "./gameScreenContent/GraveyardMenu";
import WikiMenu from "./gameScreenContent/WikiMenu";
import { Theme } from "../..";
import { StateContext } from "../../stateContext/StateContext";

export enum GameScreenMenuType {
    ChatMenu = "ChatMenu",
    RoleSpecificMenu = "RoleSpecificMenu",
    WillMenu = "WillMenu",
    PlayerListMenu = "PlayerListMenu",
    GraveyardMenu = "GraveyardMenu",
    WikiMenu = "WikiMenu",
}

export const MENU_REACT_ELEMENTS = {
    [GameScreenMenuType.ChatMenu]: ChatMenu,
    [GameScreenMenuType.PlayerListMenu]: PlayerListMenu,
    [GameScreenMenuType.RoleSpecificMenu]: AbilityMenu,
    [GameScreenMenuType.WillMenu]: WillMenu,
    [GameScreenMenuType.GraveyardMenu]: GraveyardMenu,
    [GameScreenMenuType.WikiMenu]: WikiMenu
}

export const MENU_CSS_THEMES: Record<GameScreenMenuType, Theme | null> = {
    [GameScreenMenuType.ChatMenu]: "chat-menu-colors",
    [GameScreenMenuType.PlayerListMenu]: "player-list-menu-colors",
    [GameScreenMenuType.RoleSpecificMenu]: "role-specific-colors",
    [GameScreenMenuType.WillMenu]: "will-menu-colors",
    [GameScreenMenuType.GraveyardMenu]: "graveyard-menu-colors",
    [GameScreenMenuType.WikiMenu]: "wiki-menu-colors"
}

export const MENU_TRANSLATION_KEYS: Record<GameScreenMenuType, string> = {
    [GameScreenMenuType.ChatMenu]: "menu.chat",
    [GameScreenMenuType.PlayerListMenu]: "menu.playerList",
    [GameScreenMenuType.RoleSpecificMenu]: "menu.ability",
    [GameScreenMenuType.WillMenu]: "menu.will",
    [GameScreenMenuType.GraveyardMenu]: "menu.gameMode",
    [GameScreenMenuType.WikiMenu]: "menu.wiki"
}

export const ALL_CONTENT_MENUS = Object.values(GameScreenMenuType);

export interface GameScreenMenuContext {
    closeMenu(menu: GameScreenMenuType): void;
    openMenu(menu: GameScreenMenuType, callback?: ()=>void): void;
    menusOpen(): GameScreenMenuType[];
    menuIsOpen(menu: GameScreenMenuType): boolean;
    menusAvailable(): GameScreenMenuType[]
    menuIsAvailable(menu: GameScreenMenuType): boolean;
}

export function useGameScreenMenuContext<C extends Partial<Record<GameScreenMenuType, boolean>>>(
    m: number, 
    initialOpenMenus: C,
): GameScreenMenuContext {
    const [contentMenus, setContentMenus] = useState<C>(initialOpenMenus);
    const gameState = useContext(StateContext)!;

    //call callbacks for openMenu
    const [callbacks, setCallbacks] = useState<(() => void)[]>([]);
    useEffect(() => {
        for (const callback of callbacks) {
            callback();
        }
        if (callbacks.length !== 0) {
            setCallbacks([])
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [contentMenus])

    function setContentMenu(menu: GameScreenMenuType, open: boolean) {
        const newMenus = {...contentMenus};

        if (newMenus[menu] === undefined) {
            console.log(`This screen does not have a ${menu} menu.`);
        } else {
            newMenus[menu] = open;
        }

        const menusOpen = Object.entries(newMenus).filter(([_, o])=>o === true ).map(([m, _]) => m);

        if(menusOpen.length + 1 > m && menusOpen.length > 0 && open) {
            const menuToClose = menusOpen[menusOpen.length - 1] as GameScreenMenuType;
            newMenus[menuToClose] = false;
        }

        setContentMenus(newMenus);
    }

    return {
        closeMenu(menu) {
            setContentMenu(menu, false)

            if (gameState.type === "game" && menu === GameScreenMenuType.ChatMenu){
                gameState.missedChatMessages = false;
            }
        },
        openMenu(menu, callback) {
            setContentMenu(menu, true);

            if (gameState.type === "game" && menu === GameScreenMenuType.ChatMenu){
                gameState.missedChatMessages = false;
            }

            if (callback) {
                setCallbacks(callbacks => callbacks.concat(callback))
            }
        },
        menusOpen(): GameScreenMenuType[] {
            return Object.entries(contentMenus)
                .filter(([_, open]) => open)
                .map(([menu, _]) => menu) as GameScreenMenuType[];
        },
        menuIsOpen(menu): boolean {
            return this.menusOpen().includes(menu);
        },
        menusAvailable(): GameScreenMenuType[] {
            return Object.keys(contentMenus).filter(menu => contentMenus[menu as GameScreenMenuType] !== undefined) as GameScreenMenuType[];
        },
        menuIsAvailable(menu): boolean {
            return contentMenus[menu] !== undefined
        },
    }
}

const GameScreenMenuContext = createContext<GameScreenMenuContext | undefined>(undefined)
export { GameScreenMenuContext }