import React, { ReactElement, useContext } from "react";
import GAME_MANAGER from "../../index";
import "../../index.css"
import "./startMenu.css"
import translate from "../../game/lang";
import { AnchorControllerContext } from "../Anchor";
import PlayMenu from "./PlayMenu";
import LoadingScreen from "../LoadingScreen";
import GameModesEditor from "../../components/gameModeSettings/GameModesEditor";
import Icon from "../../components/Icon";
import SettingsMenu from "../Settings";
import StandaloneWiki from "./StandaloneWiki";
import { Button } from "../../components/Button";
import Credits from "./Credits";
import StyledText from "../../components/StyledText";
import TutorialMenu from "../../tutorial/TutorialMenu";

export default function StartMenu(): ReactElement {
    const { setContent: setAnchorContent, setCoverCard } = useContext(AnchorControllerContext)!;
    return <div className="sm">
        <header>
            <h1>
                <StyledText noLinks={true}>{translate("menu.start.title")}</StyledText>
            </h1>
        </header>
        <main>
            <div className="satellite">
                <div className="left">
                    <Button onClick={async () => {
                        setAnchorContent(<LoadingScreen type="default"/>);
                        if (await GAME_MANAGER.setOutsideLobbyState()) {
                            setAnchorContent(<PlayMenu/>);
                        } else {
                            setAnchorContent(<StartMenu/>);
                        }
                    }}>
                        <Icon>play_arrow</Icon> {translate("menu.start.button.play")}
                    </Button>
                    <Button onClick={() => setAnchorContent(<TutorialMenu/>)}>
                        <Icon>school</Icon> Tutorials
                    </Button>
                    <Button onClick={() => setCoverCard(<SettingsMenu />)}>
                        <Icon>settings</Icon> {translate("menu.settings.title")}
                    </Button>
                </div>
                <div className="center"/>
                <div className="right">
                    <Button onClick={() => setCoverCard(<GameModesEditor/>)}>
                        <Icon>edit</Icon> {translate("menu.globalMenu.gameSettingsEditor")}
                    </Button>
                    <Button onClick={() => setAnchorContent(<StandaloneWiki/>)}>
                        <Icon>menu_book</Icon> {translate("menu.wiki.title")}
                    </Button>
                </div>
            </div>
        </main>
        <footer>
            <a className="button" href="https://www.github.com/ItsSammyM/mafia-rust">Github</a>
            <Button onClick={()=>{
                window.open("https://discord.gg/Vxw7gFPfJj", "_blank")
            }}>
                <Icon>public</Icon> Discord
            </Button>
            <Button onClick={()=>{setAnchorContent(<Credits/>)}}>{translate("credits")}</Button>
            <a className="button" href="https://mafia.dev.jackpapel.com">Dev (Experimental)</a>
        </footer>
    </div>
}
