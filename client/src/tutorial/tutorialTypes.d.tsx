/**
 * Tutorial System Type Definitions
 * 
 * This file defines the types for the tutorial system.
 * The tutorial system is fully client-side and simulates game scenarios.
 */

import { PhaseType, PhaseState } from "../game/gameState.d";
import { Role } from "../game/roleState.d";

/**
 * A single step in a tutorial
 */
export type TutorialStep = {
    /** Unique identifier for this step */
    id: string;
    
    /** Title displayed to the user */
    title: string;
    
    /** Instructional text for this step */
    description: string;
    
    /** Optional: Game phase this step occurs in */
    phase?: PhaseType;
    
    /** Optional: Actions that should be highlighted in the UI */
    highlightElements?: string[];
    
    /** Optional: Condition that must be met to proceed */
    completionCondition?: TutorialCompletionCondition;
    
    /** Optional: Automatic progression after a delay (in ms) */
    autoProgressDelay?: number;
    
    /** Optional: Game state changes to apply when entering this step */
    gameStateChanges?: TutorialGameStateChange[];
};

/**
 * Types of conditions that can complete a tutorial step
 */
export type TutorialCompletionCondition = 
    | { type: "click", elementId: string }
    | { type: "timer", duration: number }
    | { type: "phaseChange", targetPhase: PhaseType }
    | { type: "action", actionType: TutorialActionType }
    | { type: "manual" }; // User clicks "Next" button

/**
 * Types of actions that can be required in tutorials
 */
export type TutorialActionType =
    | { type: "selectTarget", controllerId: string }
    | { type: "submitController", controllerId: string }
    | { type: "writeWill" }
    | { type: "sendChat" }
    | { type: "vote", voteType: "nomination" | "trial" };

/**
 * Changes to apply to the game state
 */
export type TutorialGameStateChange = 
    | { type: "setPhase", phase: PhaseState }
    | { type: "addChatMessage", message: string }
    | { type: "setPlayerCount", count: number }
    | { type: "assignRole", role: Role };

/**
 * A complete tutorial definition
 */
export type Tutorial = {
    /** Unique identifier for this tutorial */
    id: string;
    
    /** Display name of the tutorial */
    name: string;
    
    /** Short description of what the tutorial teaches */
    description: string;
    
    /** Difficulty level or ordering */
    order: number;
    
    /** Whether this tutorial uses the server for game logic */
    serverBased: boolean;
    
    /** Initial game setup for the tutorial */
    initialSetup: TutorialGameSetup;
    
    /** Sequence of tutorial steps */
    steps: TutorialStep[];
};

/**
 * Initial game setup for a tutorial
 */
export type TutorialGameSetup = {
    /** Role assigned to the player */
    playerRole: Role;
    
    /** Number of players in the game */
    playerCount: number;
    
    /** Starting phase */
    startPhase: PhaseType;
    
    /** Day number to start on */
    startDay: number;
};

/**
 * Tutorial progress tracking
 */
export type TutorialProgress = {
    /** ID of the tutorial */
    tutorialId: string;
    
    /** Current step index */
    currentStepIndex: number;
    
    /** Whether the tutorial is completed */
    completed: boolean;
    
    /** Timestamp of last update */
    lastUpdated: Date;
};
