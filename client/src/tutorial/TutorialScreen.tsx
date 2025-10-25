/**
 * Tutorial Screen Component
 * 
 * Main screen displayed during a tutorial
 * Shows a simplified game interface with tutorial overlay
 */

import React, { ReactElement, useContext } from "react";
import { Button } from "../components/Button";
import Icon from "../components/Icon";
import { AnchorControllerContext } from "../menu/Anchor";
import StartMenu from "../menu/main/StartMenu";
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
                <div className="tutorial-game-placeholder">
                    <div className="tutorial-placeholder-content">
                        <Icon style={{ fontSize: "4rem" }}>lightbulb</Icon>
                        <h2>Tutorial Mode</h2>
                        <p>Follow the tutorial instructions to learn how to play</p>
                    </div>
                </div>

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
