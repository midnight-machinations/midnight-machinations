import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";
import { GameScreenMenuContext, GameScreenMenuType, MENU_REACT_ELEMENTS } from "./GameScreenMenuContext";
import React, { ReactElement, useContext } from "react";
import translate from "../../game/lang";
import { MobileContext } from "../MobileContext";

export default function GameScreenMenus(): ReactElement {
    const menuController = useContext(GameScreenMenuContext)!;
    const minSize = 10; // Percentage
    const mobile = useContext(MobileContext)!;

    // These don't add up to 100, but the panel group will fix it
    const defaultSizes = {
        [GameScreenMenuType.ChatMenu]: 35,
        [GameScreenMenuType.RoleSpecificMenu]: 15,
        [GameScreenMenuType.WillMenu]: 15,
        [GameScreenMenuType.PlayerListMenu]: 25,
        [GameScreenMenuType.GraveyardMenu]: 10,
        [GameScreenMenuType.WikiMenu]: 15,
    }

    return <PanelGroup direction="horizontal" className="content">
        {menuController.menusOpen().map((menu, index, menusOpen) => {
            const MenuElement = MENU_REACT_ELEMENTS[menu];
            return <>
                <Panel
                    className="panel"
                    minSize={minSize}
                    defaultSize={mobile===false?defaultSizes[menu]:undefined}
                    key={menu}
                >
                    <MenuElement />
                </Panel>
                {!mobile && menusOpen.some((_, i) => i > index) && <PanelResizeHandle key={index+".handle"} className="panel-handle"/>}
            </>
        })}
        {menuController.menusOpen().length === 0 && <Panel><div className="no-content">
            {translate("menu.gameScreen.noContent")}
        </div></Panel>}
    </PanelGroup>
}