/**
 * Tutorial Overlay Component
 * 
 * Displays the current tutorial step as an overlay on the game screen
 */

import React, { useEffect, useState } from "react";
import { Button } from "../components/Button";
import Icon from "../components/Icon";
import TUTORIAL_MANAGER from "./tutorialManager";
import "./tutorialOverlay.css";

export default function TutorialOverlay() {
    const [, setUpdateCounter] = useState(0);

    // Subscribe to tutorial manager updates
    useEffect(() => {
        const listener = () => {
            setUpdateCounter(prev => prev + 1);
        };

        TUTORIAL_MANAGER.addListener(listener);
        return () => {
            TUTORIAL_MANAGER.removeListener(listener);
        };
    }, []);

    const currentStep = TUTORIAL_MANAGER.getCurrentStep();
    const currentTutorial = TUTORIAL_MANAGER.getCurrentTutorial();

    if (!currentStep || !currentTutorial || !TUTORIAL_MANAGER.isActive()) {
        return null;
    }

    const currentStepIndex = TUTORIAL_MANAGER.getCurrentStepIndex();
    const totalSteps = currentTutorial.steps.length;
    const isLastStep = currentStepIndex === totalSteps - 1;
    const isFirstStep = currentStepIndex === 0;
    const isActionStep = currentStep.completionCondition?.type === "action";
    const isStepCompleted = TUTORIAL_MANAGER.isStepCompleted();

    const handleNext = () => {
        // For server-based tutorials with phase changes, advance the phase
        if (currentStep.completionCondition?.type === "phaseChange" && TUTORIAL_MANAGER.isServerBased()) {
            TUTORIAL_MANAGER.advancePhase();
        }
        
        if (!TUTORIAL_MANAGER.nextStep()) {
            // Tutorial completed
            TUTORIAL_MANAGER.endTutorial();
        }
    };

    const handlePrevious = () => {
        TUTORIAL_MANAGER.previousStep();
    };

    const handleSkip = () => {
        TUTORIAL_MANAGER.endTutorial();
    };

    return (
        <div className="tutorial-overlay">
            <div className="tutorial-backdrop" onClick={(e) => e.stopPropagation()} />
            <div className="tutorial-panel">
                <div className="tutorial-header">
                    <h2>{currentStep.title}</h2>
                    <button className="tutorial-close" onClick={handleSkip} aria-label="Skip Tutorial">
                        <Icon>close</Icon>
                    </button>
                </div>
                
                <div className="tutorial-content">
                    <p>{currentStep.description}</p>
                    {isActionStep && !isStepCompleted && (
                        <div className="tutorial-action-required">
                            <Icon>touch_app</Icon>
                            <span>Action required to continue</span>
                        </div>
                    )}
                    {isActionStep && isStepCompleted && (
                        <div className="tutorial-action-completed">
                            <Icon>check_circle</Icon>
                            <span>Action completed!</span>
                        </div>
                    )}
                </div>

                <div className="tutorial-footer">
                    <div className="tutorial-progress">
                        Step {currentStepIndex + 1} of {totalSteps}
                    </div>
                    
                    <div className="tutorial-actions">
                        {!isFirstStep && !isActionStep && (
                            <Button onClick={handlePrevious}>
                                <Icon>arrow_back</Icon> Previous
                            </Button>
                        )}
                        
                        {isLastStep ? (
                            <Button onClick={handleSkip}>
                                <Icon>check</Icon> Finish
                            </Button>
                        ) : isActionStep ? (
                            <Button onClick={handleNext} disabled={!isStepCompleted}>
                                {isStepCompleted ? "Continue" : "Waiting for action..."} <Icon>arrow_forward</Icon>
                            </Button>
                        ) : (
                            <Button onClick={handleNext}>
                                Next <Icon>arrow_forward</Icon>
                            </Button>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
