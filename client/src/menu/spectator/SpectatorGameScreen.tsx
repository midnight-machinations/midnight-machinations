import React, { ReactElement, useContext } from "react";
import "../game/gameScreen.css"
import HeaderMenu, { MenuButtons } from "../game/HeaderMenu";
import { GameScreenMenus } from "../game/GameScreen";
import { MobileContext } from "../Anchor";
import { loadSettingsParsed } from "../../game/localStorage";
import { MenuControllerContext, useMenuController } from "../game/menuController";
import GAME_MANAGER from "../..";

export default function SpectatorGameScreen(): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    const contentController = useMenuController(
        maxMenus,
        menuOrder.filter((kvp)=>kvp[1]).map((kvp)=>kvp[0]),
        GAME_MANAGER.state.stateType==="game"&&GAME_MANAGER.state.clientState.type==="spectator"
    );


    return (
        <MenuControllerContext.Provider value={contentController}>
            <div className="game-screen spectator-game-screen">
                <div className="header">
                    <HeaderMenu chatMenuNotification={false}/>
                </div>
                <GameScreenMenus />
                {mobile === true && <MenuButtons chatMenuNotification={false}/>}
            </div>
        </MenuControllerContext.Provider>
    );
    
}