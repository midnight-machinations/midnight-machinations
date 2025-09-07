import React from 'react';
import BoysKissing from './BoysKissing';
import { useBoysKissing } from './useBoysKissing';
import './boysKissingOverlay.css';

interface BoysKissingOverlayProps {
    playerNames?: string[];
    enabled?: boolean;
    onEventTriggered?: (event: any) => void;
}

export default function BoysKissingOverlay({ 
    playerNames, 
    enabled = true,
    onEventTriggered 
}: BoysKissingOverlayProps) {
    const { 
        currentEvent, 
        isEventActive, 
        triggerBoysKissingEvent,
        clearCurrentEvent,
        getKissStats
    } = useBoysKissing({
        enableRandomEvents: enabled,
        eventProbability: 0.15, // 15% chance per minute
        playerNames
    });

    const stats = getKissStats();

    const handleManualKiss = () => {
        const event = triggerBoysKissingEvent();
        if (onEventTriggered) {
            onEventTriggered(event);
        }
    };

    if (!enabled) return null;

    return (
        <>
            {/* Stats display */}
            <div className="boys-kissing-stats">
                <button 
                    className="kiss-trigger-btn"
                    onClick={handleManualKiss}
                    disabled={isEventActive}
                    title="Trigger a romantic moment! 💕"
                >
                    💋 Kiss
                </button>
                <div className="kiss-counter">
                    💖 {stats.totalKisses} kisses total
                </div>
                {stats.favoriteCouple && (
                    <div className="favorite-couple">
                        👬 {stats.favoriteCouple.names.join(' & ')} ({stats.favoriteCouple.count}x)
                    </div>
                )}
            </div>

            {/* Active event overlay */}
            {isEventActive && currentEvent && (
                <div className="boys-kissing-overlay-backdrop">
                    <div className="boys-kissing-overlay-content">
                        <button 
                            className="close-overlay-btn"
                            onClick={clearCurrentEvent}
                            aria-label="Close romantic moment"
                        >
                            ✕
                        </button>
                        <BoysKissing
                            boy1Name={currentEvent.boy1}
                            boy2Name={currentEvent.boy2}
                            onKissComplete={clearCurrentEvent}
                        />
                    </div>
                </div>
            )}
        </>
    );
}
