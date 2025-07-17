import React, { ReactElement, useContext } from "react";
import "./wikiMenu.css"
import translate from "../../../game/lang";
import Wiki from "../../../wiki/Wiki";
import { getAllRoles } from "../../../stateContext/stateType/roleListState";
import { GameScreenMenuType } from "../GameScreenMenuContext";
import GameScreenMenuTab from "../GameScreenMenuTab";
import { MODIFIERS, ModifierType } from "../../../stateContext/stateType/modifiersState";
import { StateContext } from "../../../stateContext/StateContext";

export default function WikiMenu(): ReactElement {
    const enabledRoles = useContext(StateContext)?.enabledRoles??getAllRoles();
    const enabledModifiers = useContext(StateContext)?.enabledModifiers??MODIFIERS as any as ModifierType[];

    return <div className="wiki-menu wiki-menu-colors">
        <GameScreenMenuTab close={GameScreenMenuType.WikiMenu} helpMenu={null}>{translate("menu.wiki.title")}</GameScreenMenuTab>
        
        <div className="wiki-menu-search">
            <Wiki enabledRoles={enabledRoles} enabledModifiers={enabledModifiers}/>
        </div>
    </div>
}