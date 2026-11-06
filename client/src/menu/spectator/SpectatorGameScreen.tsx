import { type ReactElement, useContext } from "react";
import "../game/gameScreen.css";
import { loadSettingsParsed } from "../../game/localStorage";
import { MobileContext } from "../Anchor";
import { GameScreenMenus, type MenuController, MenuControllerContext, useMenuController } from "../game/GameScreen";
import HeaderMenu, { MenuButtons } from "../game/HeaderMenu";

let CONTENT_CONTROLLER: MenuController | undefined;

export function getSpectatorScreenContentController(): MenuController | undefined {
    return CONTENT_CONTROLLER;
}

export default function SpectatorGameScreen(): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    const contentController = useMenuController(
        maxMenus,
        Object.fromEntries(menuOrder),
        () => CONTENT_CONTROLLER!,
        (contentController) => (CONTENT_CONTROLLER = contentController),
    );

    return (
        <MenuControllerContext.Provider value={contentController}>
            <div className="game-screen spectator-game-screen">
                <div className="header">
                    <HeaderMenu chatMenuNotification={false} />
                </div>
                <GameScreenMenus />
                {mobile === true && <MenuButtons chatMenuNotification={false} />}
            </div>
        </MenuControllerContext.Provider>
    );
}
