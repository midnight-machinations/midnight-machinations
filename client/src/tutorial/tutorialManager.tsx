/**
 * Tutorial Manager
 * 
 * Manages tutorial state, progression, and execution.
 * This is a singleton that coordinates all tutorial-related functionality.
 * Supports both client-side simulation and server-based interactive tutorials.
 */

import { Tutorial, TutorialStep, TutorialProgress, TutorialActionType } from "./tutorialTypes.d";
import GAME_MANAGER from "../index";
import { createGameState, createPlayer, createPlayerGameState } from "../game/gameState";
import GameState from "../game/gameState.d";

class TutorialManager {
    private currentTutorial: Tutorial | null = null;
    private currentStepIndex: number = 0;
    private listeners: Array<() => void> = [];
    private active: boolean = false;
    private originalState: any = null;
    private serverBased: boolean = false;
    private stepCompleted: boolean = false;

    /**
     * Check if the current tutorial is server-based
     */
    isServerBased(): boolean {
        return this.serverBased;
    }

    /**
     * Mark the current step as completed (used for action-based steps)
     */
    markStepCompleted(): void {
        this.stepCompleted = true;
        this.notifyListeners();
    }

    /**
     * Check if current step is completed
     */
    isStepCompleted(): boolean {
        return this.stepCompleted;
    }

    /**
     * Validate if a game action completes the current step
     */
    validateAction(actionType: TutorialActionType): boolean {
        const step = this.getCurrentStep();
        if (!step || !step.completionCondition) {
            return false;
        }

        if (step.completionCondition.type === "action") {
            // Compare action types
            const required = step.completionCondition.actionType;
            if (required.type === actionType.type) {
                this.markStepCompleted();
                return true;
            }
        }

        return false;
    }

    /**
     * Advance to next phase (for server-based tutorials)
     */
    advancePhase(): void {
        if (this.serverBased && GAME_MANAGER.state.stateType === "game") {
            GAME_MANAGER.server.sendPacket({ type: "tutorialAdvancePhase" });
        }
    }

    /**
     * Create a simulated game state for the tutorial
     */
    private createTutorialGameState(tutorial: Tutorial): GameState {
        const gameState = createGameState();
        const setup = tutorial.initialSetup;
        
        // Set up basic game info
        gameState.roomCode = 99999;
        gameState.lobbyName = `Tutorial: ${tutorial.name}`;
        gameState.initialized = true;
        gameState.myId = 0;
        
        // Create simulated players
        gameState.players = [];
        const playerNames = [
            "You",
            "Alice",
            "Bob", 
            "Charlie",
            "Diana",
            "Eve",
            "Frank",
            "Grace",
            "Henry",
            "Ivy",
            "Jack",
            "Kate",
            "Leo",
            "Maya",
            "Noah"
        ];
        
        for (let i = 0; i < setup.playerCount; i++) {
            gameState.players.push(createPlayer(playerNames[i], i));
        }
        
        // Set phase state - handle different phase types properly
        switch (setup.startPhase) {
            case "briefing":
                gameState.phaseState = { type: "briefing" };
                break;
            case "dusk":
                gameState.phaseState = { type: "dusk" };
                break;
            case "night":
                gameState.phaseState = { type: "night" };
                break;
            case "obituary":
                gameState.phaseState = { type: "obituary" };
                break;
            case "discussion":
                gameState.phaseState = { type: "discussion" };
                break;
            case "nomination":
                gameState.phaseState = { type: "nomination", trialsLeft: 3 };
                break;
            case "adjournment":
                gameState.phaseState = { type: "adjournment", trialsLeft: 3 };
                break;
            case "testimony":
                gameState.phaseState = { type: "testimony", playerOnTrial: 1, trialsLeft: 3 };
                break;
            case "judgement":
                gameState.phaseState = { type: "judgement", playerOnTrial: 1, trialsLeft: 3 };
                break;
            case "finalWords":
                gameState.phaseState = { type: "finalWords", playerOnTrial: 1 };
                break;
            default:
                gameState.phaseState = { type: "briefing" };
        }
        
        gameState.dayNumber = setup.startDay;
        gameState.timeLeftMs = null;
        gameState.ticking = false;
        
        // Set up player state
        const playerState = createPlayerGameState();
        playerState.myIndex = 0;
        playerState.myRole = setup.playerRole;
        gameState.clientState = playerState;
        
        return gameState;
    }

    /**
     * Start a tutorial
     * @param tutorial The tutorial to start
     * @param serverBased Whether to use server-based game (true for interactive tutorials)
     */
    startTutorial(tutorial: Tutorial, serverBased: boolean = false): void {
        this.serverBased = serverBased;
        this.currentStepIndex = 0;
        this.stepCompleted = false;

        if (serverBased) {
            // For server-based tutorials, we don't create a simulated state
            // The server will handle game creation
            this.originalState = GAME_MANAGER.state;
        } else {
            // Save the original game manager state for client-side simulation
            this.originalState = GAME_MANAGER.state;
            
            // Create and set the tutorial game state
            const tutorialState = this.createTutorialGameState(tutorial);
            GAME_MANAGER.state = tutorialState;
        }
        
        this.currentTutorial = tutorial;
        this.active = true;
        this.notifyListeners();
    }

    /**
     * Get the current tutorial
     */
    getCurrentTutorial(): Tutorial | null {
        return this.currentTutorial;
    }

    /**
     * Get the current step
     */
    getCurrentStep(): TutorialStep | null {
        if (!this.currentTutorial || !this.active) {
            return null;
        }
        return this.currentTutorial.steps[this.currentStepIndex] ?? null;
    }

    /**
     * Get the current step index
     */
    getCurrentStepIndex(): number {
        return this.currentStepIndex;
    }

    /**
     * Check if a tutorial is active
     */
    isActive(): boolean {
        return this.active;
    }

    /**
     * Advance to the next step
     */
    nextStep(): boolean {
        if (!this.currentTutorial || !this.active) {
            return false;
        }

        if (this.currentStepIndex < this.currentTutorial.steps.length - 1) {
            this.currentStepIndex++;
            this.stepCompleted = false;
            this.notifyListeners();
            return true;
        }

        // Tutorial completed
        return false;
    }

    /**
     * Go back to the previous step
     */
    previousStep(): boolean {
        if (!this.currentTutorial || !this.active) {
            return false;
        }

        if (this.currentStepIndex > 0) {
            this.currentStepIndex--;
            this.stepCompleted = false;
            this.notifyListeners();
            return true;
        }

        return false;
    }

    /**
     * End the current tutorial
     */
    endTutorial(): void {
        // Restore the original game manager state
        if (this.originalState !== null && !this.serverBased) {
            GAME_MANAGER.state = this.originalState;
            this.originalState = null;
        }
        
        this.currentTutorial = null;
        this.currentStepIndex = 0;
        this.active = false;
        this.serverBased = false;
        this.stepCompleted = false;
        this.notifyListeners();
    }

    /**
     * Add a listener for tutorial state changes
     */
    addListener(listener: () => void): void {
        this.listeners.push(listener);
    }

    /**
     * Remove a listener
     */
    removeListener(listener: () => void): void {
        const index = this.listeners.indexOf(listener);
        if (index !== -1) {
            this.listeners.splice(index, 1);
        }
    }

    /**
     * Notify all listeners of state changes
     */
    private notifyListeners(): void {
        this.listeners.forEach(listener => listener());
    }

    /**
     * Check if a step's completion condition is met
     */
    checkStepCompletion(step: TutorialStep, context?: any): boolean {
        if (!step.completionCondition) {
            return true; // Manual progression
        }

        switch (step.completionCondition.type) {
            case "manual":
                return true;
            case "timer":
                // Timer completion is handled externally
                return false;
            case "click":
            case "phaseChange":
            case "action":
                // These require external triggers
                return false;
            default:
                return false;
        }
    }

    /**
     * Get tutorial progress for saving
     */
    getProgress(): TutorialProgress | null {
        if (!this.currentTutorial) {
            return null;
        }

        return {
            tutorialId: this.currentTutorial.id,
            currentStepIndex: this.currentStepIndex,
            completed: this.currentStepIndex >= this.currentTutorial.steps.length - 1,
            lastUpdated: new Date()
        };
    }
}

// Export singleton instance
const TUTORIAL_MANAGER = new TutorialManager();
export default TUTORIAL_MANAGER;
