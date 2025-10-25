/**
 * Tutorial Manager
 * 
 * Manages tutorial state, progression, and execution.
 * This is a singleton that coordinates all tutorial-related functionality.
 */

import { Tutorial, TutorialStep, TutorialProgress } from "./tutorialTypes.d";

class TutorialManager {
    private currentTutorial: Tutorial | null = null;
    private currentStepIndex: number = 0;
    private listeners: Array<() => void> = [];
    private active: boolean = false;

    /**
     * Start a tutorial
     */
    startTutorial(tutorial: Tutorial): void {
        this.currentTutorial = tutorial;
        this.currentStepIndex = 0;
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
            this.notifyListeners();
            return true;
        }

        return false;
    }

    /**
     * End the current tutorial
     */
    endTutorial(): void {
        this.currentTutorial = null;
        this.currentStepIndex = 0;
        this.active = false;
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
