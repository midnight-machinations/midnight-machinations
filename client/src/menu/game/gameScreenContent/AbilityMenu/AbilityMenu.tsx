import React, { ReactElement, useContext } from "react";
import translate from "../../../../game/lang";
import GenericAbilityMenu from "./GenericAbilityMenu";
import "./abilityMenu.css";
import RoleSpecificSection from "./RoleSpecific";
import GameScreenMenuTab from "../../GameScreenMenuTab";
import { GameScreenMenuType } from "../../GameScreenMenuContext";
import { StateContext } from "../../../../stateContext/StateContext";

export default function AbilityMenu(): ReactElement {
    const IsSpectator = useContext(StateContext)!.clientState.type === "spectator";

    return <div className="ability-menu role-specific-colors">
        <GameScreenMenuTab close={GameScreenMenuType.RoleSpecificMenu} helpMenu={"standard/abilityMenu"}>
            {translate("menu.ability.title")}
        </GameScreenMenuTab>
        {!IsSpectator &&
            <div className="abilities">
                <RoleSpecificSection/>
                <GenericAbilityMenu/>
            </div>
        }
    </div>
    
}