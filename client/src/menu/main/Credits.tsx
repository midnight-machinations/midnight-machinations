import React, { useContext } from "react";
import { ReactElement } from "react";
import translate from "../../game/lang";
import "./credits.css";
import StartMenu from "./StartMenu";
import { AppContext } from "../AppContext";

export default function Credits(): ReactElement {
    const anchorContext = useContext(AppContext)!;

    return <div className="credits-menu">
        <button onClick={() => {
            anchorContext.setContent({type:"main"});
        }}>{translate("menu.globalMenu.quitToMenu")}</button>
        <div>
            <h3>{translate("leadDevelopers")}</h3>
            <p>
                <span>Jack Papel (<a href="https://www.jackpapel.com">Website</a>)</span>
                <span>Sammy</span>
            </p>
            <h3>{translate("otherContributors")}</h3>
            <p>
                <span>copop22</span>
                <span>Gabriel Arias</span>
                <span>Thomas Berrios</span>
                <span>Willow Thompson</span>
            </p>
            <h3>{translate("playTesters")}</h3>
            <p>
                <span>Alex Eng</span>
                <span>Bit Haag</span>
                <span>copop22</span>
                <span>Firefly707</span>
                <span>Gabriel Arias</span>
                <span>Genevieve Rolnick</span>
                <span>Jamin Chen</span>
                <span>Kate Horne</span>
                <span>Marcus Moher</span>
                <span>Myanmar (Mascot :3)</span>
                <span>Vigil</span>
                <span>Willow Thompson</span>
                <span>Zach Feeney</span>
            </p>
        </div>
    </div>
}