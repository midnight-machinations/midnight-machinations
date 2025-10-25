/**
 * Tutorial Menu Component
 * 
 * Displays available tutorials and allows users to start them
 */

import React, { ReactElement, useContext } from "react";
import { Button } from "../components/Button";
import Icon from "../components/Icon";
import { AnchorControllerContext } from "../menu/Anchor";
import StartMenu from "../menu/main/StartMenu";
import GAME_MANAGER from "../index";
import TUTORIAL_MANAGER from "./tutorialManager";
import { ALL_TUTORIALS } from "./tutorials";
import TutorialScreen from "./TutorialScreen";
import LoadingScreen from "../menu/LoadingScreen";
import "./tutorialMenu.css";

export default function TutorialMenu(): ReactElement {
    const { setContent } = useContext(AnchorControllerContext)!;

    const startTutorial = async (tutorialId: string) => {
        const tutorial = ALL_TUTORIALS.find(t => t.id === tutorialId);
        if (!tutorial) return;

        if (tutorial.serverBased) {
            // For server-based tutorials, connect to server and create a tutorial game
            setContent(<LoadingScreen type="default"/>);
            
            if (!await GAME_MANAGER.setOutsideLobbyState()) {
                setContent(<StartMenu/>);
                return;
            }

            // Host a tutorial game
            const success = await GAME_MANAGER.sendHostPacket();
            if (!success) {
                setContent(<StartMenu/>);
                return;
            }

            // Wait for lobby state, then configure tutorial settings
            // Note: The game manager will handle the lobby setup
            // We just need to mark this as a tutorial
            TUTORIAL_MANAGER.startTutorial(tutorial, true);
            // The user will be in lobby and needs to start the game from there
            // For now, just show the tutorial screen which will show the lobby
            setContent(<TutorialScreen />);
        } else {
            // Client-side simulation tutorial
            TUTORIAL_MANAGER.startTutorial(tutorial, false);
            setContent(<TutorialScreen />);
        }
    };

    return (
        <div className="tutorial-menu">
            <header className="tutorial-menu-header">
                <h1>
                    <Icon>school</Icon>
                    Tutorials
                </h1>
                <p>Learn how to play Midnight Machinations through interactive tutorials</p>
            </header>

            <main className="tutorial-menu-content">
                <div className="tutorial-list">
                    {ALL_TUTORIALS.sort((a, b) => a.order - b.order).map(tutorial => (
                        <div key={tutorial.id} className="tutorial-card">
                            <div className="tutorial-card-header">
                                <h2>{tutorial.name}</h2>
                                <span className="tutorial-order">#{tutorial.order}</span>
                            </div>
                            <p className="tutorial-description">{tutorial.description}</p>
                            <div className="tutorial-card-footer">
                                <Button onClick={() => startTutorial(tutorial.id)}>
                                    <Icon>play_arrow</Icon>
                                    Start Tutorial
                                </Button>
                            </div>
                        </div>
                    ))}
                </div>
            </main>

            <footer className="tutorial-menu-footer">
                <Button onClick={() => setContent(<StartMenu/>)}>
                    <Icon>arrow_back</Icon>
                    Back to Main Menu
                </Button>
            </footer>
        </div>
    );
}
