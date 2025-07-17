import React, { ReactElement, useContext, useEffect } from "react";
import HeaderMenu, { MenuButtons } from "./HeaderMenu";
import { DEV_ENV } from "../..";
import "../../index.css";
import "./gameScreen.css";
import { loadSettingsParsed } from "../../game/localStorage";
import { GameScreenMenuContext, GameScreenMenuType, useGameScreenMenuContext } from "./GameScreenMenuContext";
import GameScreenMenus from "./GameScreenMenus";
import { MobileContext } from "../MobileContext";
import { TickContext, useTickContext } from "./TickContext";

export default function GameScreen(props: {isSpectator: boolean}): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    let menusOpen: [GameScreenMenuType, boolean | undefined][];
    if(props.isSpectator){
        menusOpen = [
            [GameScreenMenuType.WikiMenu, undefined ],
            [GameScreenMenuType.GraveyardMenu, maxMenus > 2 ],
            [GameScreenMenuType.PlayerListMenu, maxMenus > 1 ],
            [GameScreenMenuType.ChatMenu, true ],
            [GameScreenMenuType.WillMenu, undefined ],
            [GameScreenMenuType.RoleSpecificMenu, undefined ],
        ];
    }else{
        menusOpen = [
            [GameScreenMenuType.WikiMenu, mobile ? undefined : false ],
            [GameScreenMenuType.GraveyardMenu, maxMenus > 4 ],
            [GameScreenMenuType.PlayerListMenu, maxMenus > 1 ],
            [GameScreenMenuType.ChatMenu, true ],
            [GameScreenMenuType.WillMenu, maxMenus > 3 ],
            [GameScreenMenuType.RoleSpecificMenu, maxMenus > 2 ],
        ];
    }

    menusOpen.sort((a, b) => menuOrder.indexOf(a[0]) - menuOrder.indexOf(b[0]))

    const menuController = useGameScreenMenuContext(maxMenus, Object.fromEntries(menusOpen));
    

    useEffect(() => {
        const onBeforeUnload = (e: BeforeUnloadEvent) => {
            if (!DEV_ENV) e.preventDefault()
        };

        window.addEventListener("beforeunload", onBeforeUnload);
        return () => window.removeEventListener("beforeunload", onBeforeUnload);
    }, []);
    
    const tickContext = useTickContext();

    return <TickContext.Provider value={tickContext}>
            <GameScreenMenuContext.Provider value={menuController}>
                <div className="game-screen">
                    <div className="header">
                        <HeaderMenu/>
                    </div>
                    <GameScreenMenus/>
                    {mobile && <MenuButtons/>}
                </div>
            </GameScreenMenuContext.Provider>
    </TickContext.Provider>
}
