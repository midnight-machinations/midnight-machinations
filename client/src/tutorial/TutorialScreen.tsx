/**
 * Tutorial Screen Component
 * 
 * Main screen displayed during a tutorial
 * Shows the actual game interface with tutorial overlay
 */

import React, { ReactElement, useContext } from "react";
import { Button } from "../components/Button";
import Icon from "../components/Icon";
import { AnchorControllerContext } from "../menu/Anchor";
import StartMenu from "../menu/main/StartMenu";
import GameScreen from "../menu/game/GameScreen";
import TutorialOverlay from "./TutorialOverlay";
import TUTORIAL_MANAGER from "./tutorialManager";
import "./tutorialScreen.css";

export default function TutorialScreen(): ReactElement {
    const { setContent } = useContext(AnchorControllerContext)!;

    const exitTutorial = () => {
        TUTORIAL_MANAGER.endTutorial();
        setContent(<StartMenu />);
    };

    const tutorial = TUTORIAL_MANAGER.getCurrentTutorial();

    if (!tutorial) {
        setContent(<StartMenu />);
        return <></>;
    }

    return (
        <div className="tutorial-screen">
            <div className="tutorial-game-view">
                <GameScreen />
                
                <div className="tutorial-controls">
                    <Button onClick={exitTutorial} className="tutorial-exit-button">
                        <Icon>close</Icon>
                        Exit Tutorial
                    </Button>
                </div>
            </div>

            <TutorialOverlay />
        </div>
    );
}
