import { useState, useEffect, useCallback } from 'react';
import translate from '../game/lang';

interface BoysKissingEvent {
    id: string;
    boy1: string;
    boy2: string;
    timestamp: number;
    type: 'cute' | 'romantic' | 'wholesome' | 'passionate';
}

interface UseBoysKissingOptions {
    enableRandomEvents?: boolean;
    eventProbability?: number; // 0-1, chance per minute
    playerNames?: string[];
}

export function useBoysKissing(options: UseBoysKissingOptions = {}) {
    const {
        enableRandomEvents = true,
        eventProbability = 0.1,
        playerNames = ['Alex', 'Jamie', 'Dylan', 'River', 'Casey', 'Jordan']
    } = options;

    const [currentEvent, setCurrentEvent] = useState<BoysKissingEvent | null>(null);
    const [eventHistory, setEventHistory] = useState<BoysKissingEvent[]>([]);
    const [isEventActive, setIsEventActive] = useState(false);

    const getRandomPlayer = useCallback(() => {
        return playerNames[Math.floor(Math.random() * playerNames.length)];
    }, [playerNames]);

    const getRandomKissType = useCallback((): BoysKissingEvent['type'] => {
        const types: BoysKissingEvent['type'][] = ['cute', 'romantic', 'wholesome', 'passionate'];
        return types[Math.floor(Math.random() * types.length)];
    }, []);

    const triggerBoysKissingEvent = useCallback((boy1?: string, boy2?: string, type?: BoysKissingEvent['type']) => {
        const selectedBoy1 = boy1 || getRandomPlayer();
        let selectedBoy2 = boy2 || getRandomPlayer();
        
        // Ensure we don't have the same boy kissing himself
        while (selectedBoy2 === selectedBoy1 && playerNames.length > 1) {
            selectedBoy2 = getRandomPlayer();
        }

        const event: BoysKissingEvent = {
            id: `kiss-${Date.now()}`,
            boy1: selectedBoy1,
            boy2: selectedBoy2,
            timestamp: Date.now(),
            type: type || getRandomKissType()
        };

        setCurrentEvent(event);
        setIsEventActive(true);
        setEventHistory(prev => [...prev, event]);

        // Auto-clear event after 5 seconds
        setTimeout(() => {
            setIsEventActive(false);
            setCurrentEvent(null);
        }, 5000);

        return event;
    }, [getRandomPlayer, getRandomKissType, playerNames.length]);

    // Random event generator
    useEffect(() => {
        if (!enableRandomEvents) return;

        const interval = setInterval(() => {
            if (Math.random() < eventProbability && !isEventActive) {
                triggerBoysKissingEvent();
            }
        }, 60000); // Check every minute

        return () => clearInterval(interval);
    }, [enableRandomEvents, eventProbability, isEventActive, triggerBoysKissingEvent]);

    const getChatMessage = useCallback((event: BoysKissingEvent) => {
        const messageKey = `chatMessage.boysKissing.${event.type}`;
        return translate(messageKey, event.boy1, event.boy2);
    }, []);

    const getKissStats = useCallback(() => {
        const totalKisses = eventHistory.length;
        const kissesToday = eventHistory.filter(
            event => Date.now() - event.timestamp < 24 * 60 * 60 * 1000
        ).length;
        
        const favoriteCouple = eventHistory.reduce((acc, event) => {
            const coupleKey = [event.boy1, event.boy2].sort().join('-');
            acc[coupleKey] = (acc[coupleKey] || 0) + 1;
            return acc;
        }, {} as Record<string, number>);

        const mostPopularCouple = Object.entries(favoriteCouple)
            .sort(([, a], [, b]) => b - a)[0];

        return {
            totalKisses,
            kissesToday,
            favoriteCouple: mostPopularCouple ? {
                names: mostPopularCouple[0].split('-'),
                count: mostPopularCouple[1]
            } : null
        };
    }, [eventHistory]);

    return {
        currentEvent,
        eventHistory,
        isEventActive,
        triggerBoysKissingEvent,
        getChatMessage,
        getKissStats,
        clearCurrentEvent: () => {
            setCurrentEvent(null);
            setIsEventActive(false);
        }
    };
}

export default useBoysKissing;
