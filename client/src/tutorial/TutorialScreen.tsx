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
import LobbyMenu from "../menu/lobby/LobbyMenu";
import GAME_MANAGER from "../index";
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

    // For server-based tutorials, check if we're in lobby or game state
    const isServerBased = TUTORIAL_MANAGER.isServerBased();
    const currentState = GAME_MANAGER.state.stateType;

    // If server-based and still in lobby, show lobby screen
    if (isServerBased && currentState === "lobby") {
        return (
            <div className="tutorial-screen">
                <div className="tutorial-game-view">
                    <LobbyMenu />
                    
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

    // For game state or client-side tutorials, show game screen
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
