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
            // For server-based tutorials, connect to server and automatically start the game
            setContent(<LoadingScreen type="default"/>);
            
            if (!await GAME_MANAGER.setOutsideLobbyState()) {
                setContent(<StartMenu/>);
                return;
            }

            // Host a tutorial game
            const hostSuccess = await GAME_MANAGER.sendHostPacket();
            if (!hostSuccess) {
                setContent(<StartMenu/>);
                return;
            }

            // Wait a moment for lobby state to initialize
            await new Promise(resolve => setTimeout(resolve, 100));

            // Configure the lobby for the tutorial
            if (GAME_MANAGER.state.stateType === "lobby") {
                // Set up the tutorial game configuration
                const setup = tutorial.initialSetup;
                
                // Enable tutorial mode - this tells the server to add bot players
                GAME_MANAGER.server.sendPacket({
                    type: "setTutorialMode",
                    tutorialMode: true
                });
                
                // Set fixed seed for reproducible tutorials
                GAME_MANAGER.server.sendPacket({
                    type: "setRandomSeed",
                    randomSeed: 12345
                });

                // Configure role list for tutorial
                // For detective tutorial, set first role to detective, rest to any
                const roleList: any[] = [];
                if (setup.playerRole === "detective") {
                    // First role is detective (for the player)
                    roleList.push([{ roles: { type: "role", role: "detective" } }]);
                    // Add 8 more "any" roles for the bots
                    for (let i = 0; i < 8; i++) {
                        roleList.push([{ roles: { type: "roleSet", roleSet: "any" } }]);
                    }
                }

                GAME_MANAGER.server.sendPacket({
                    type: "setRoleList",
                    roleList: roleList
                });

                // Enable all roles so any role can be selected
                GAME_MANAGER.server.sendPacket({
                    type: "setEnabledRoles",
                    roles: ["jailor", "detective", "lookout", "spy", "tracker", "philosopher", "psychic", 
                            "auditor", "snoop", "gossip", "doctor", "bodyguard", "cop", 
                            "bouncer", "engineer", "armorsmith", "vigilante", "veteran", "marksman", 
                            "deputy", "escort", "medium", "mayor", 
                            "transporter", "godfather", "mafioso", "hypnotist", "blackmailer", 
                            "informant", "consort", "mortician", "framer", "forger", "witch", 
                            "jester", "politician", "wildcard", 
                            "scarecrow", "martyr", "steward", "kira", "revolutionary", 
                            "arsonist", "werewolf", "serialKiller"]
                });
                
                // Start the game automatically (server will add bots)
                const startSuccess = await GAME_MANAGER.sendStartGamePacket();
                if (!startSuccess) {
                    setContent(<StartMenu/>);
                    return;
                }
            }

            // Mark this as a tutorial
            TUTORIAL_MANAGER.startTutorial(tutorial, true);
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
