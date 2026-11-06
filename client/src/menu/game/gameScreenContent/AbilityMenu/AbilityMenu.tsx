import type { ReactElement } from "react";
import { useGameState } from "../../../../components/useHooks";
import translate from "../../../../game/lang";
import { ContentMenu, ContentTab } from "../../GameScreen";
import GenericAbilityMenu from "./GenericAbilityMenu";
import "./abilityMenu.css";
import RoleSpecificSection from "./RoleSpecific";

export default function AbilityMenu(): ReactElement {
    const mySpectator = useGameState(
        (gameState) => gameState.clientState.type === "spectator",
        ["gamePlayers", "acceptJoin"],
    )!;

    return (
        <div className="ability-menu role-specific-colors">
            <ContentTab close={ContentMenu.RoleSpecificMenu}>{translate("menu.ability.title")}</ContentTab>
            {!mySpectator && (
                <div>
                    <RoleSpecificSection />
                    <GenericAbilityMenu />
                </div>
            )}
        </div>
    );
}
