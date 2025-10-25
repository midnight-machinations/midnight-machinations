/**
 * Tutorial Definitions
 * 
 * This file contains all available tutorials.
 * New tutorials can be easily added by creating a new Tutorial object.
 */

import { Tutorial } from "./tutorialTypes.d";

/**
 * Basic Gameplay Tutorial
 * Teaches the fundamentals of how the game works
 */
export const basicGameplayTutorial: Tutorial = {
    id: "basic-gameplay",
    name: "Basic Gameplay",
    description: "Learn the fundamentals of Midnight Machinations",
    order: 1,
    initialSetup: {
        playerRole: "villager",
        playerCount: 7,
        startPhase: "briefing",
        startDay: 1
    },
    steps: [
        {
            id: "welcome",
            title: "Welcome to Midnight Machinations!",
            description: "This tutorial will teach you the basics of playing the game. Midnight Machinations is a social deduction game where players work together to find the evil players among them.",
            completionCondition: { type: "manual" }
        },
        {
            id: "game-phases",
            title: "Game Phases",
            description: "The game is divided into different phases that repeat each day: Briefing (day 1 only), Discussion, Nomination, Judgement, Testimony, Final Words, Dusk, and Night. Each phase has different actions available.",
            completionCondition: { type: "manual" }
        },
        {
            id: "your-role",
            title: "Your Role",
            description: "You have been assigned the Villager role. As a Villager, you are a member of the Town faction. Your goal is to eliminate all members of the Mafia and other evil factions.",
            completionCondition: { type: "manual" }
        },
        {
            id: "day-phase",
            title: "Day Phases",
            description: "During the day (Discussion, Nomination, and Voting phases), players can talk to each other, share information, and vote to put players on trial. Pay attention to what others say and look for suspicious behavior.",
            completionCondition: { type: "manual" }
        },
        {
            id: "night-phase",
            title: "Night Phase",
            description: "During the Night phase, most roles use their special abilities. Some roles can investigate others, some can protect players, and some can attack. The Villager doesn't have a night ability, so you'll just wait for the next day.",
            completionCondition: { type: "manual" }
        },
        {
            id: "chat",
            title: "Communication",
            description: "The chat is your main tool for sharing information and coordinating with other players. Use it to discuss suspicions, share results from night abilities, and work together to find the Mafia.",
            completionCondition: { type: "manual" }
        },
        {
            id: "voting",
            title: "Voting and Trials",
            description: "During the Nomination phase, players can nominate others for trial. If enough players vote for the nomination, that player goes on trial. During Judgement, everyone votes guilty or innocent to determine if the player should be eliminated.",
            completionCondition: { type: "manual" }
        },
        {
            id: "win-condition",
            title: "Winning the Game",
            description: "The Town wins when all members of the Mafia and other evil factions are eliminated. The Mafia wins when they equal or outnumber the Town. Pay attention to the player list and think carefully about who might be evil!",
            completionCondition: { type: "manual" }
        },
        {
            id: "conclusion",
            title: "Tutorial Complete!",
            description: "You now know the basics of Midnight Machinations! The best way to learn is to play. Try joining a game and putting these concepts into practice. Good luck!",
            completionCondition: { type: "manual" }
        }
    ]
};

/**
 * Array of all available tutorials
 */
export const ALL_TUTORIALS: Tutorial[] = [
    basicGameplayTutorial
];

/**
 * Get a tutorial by ID
 */
export function getTutorialById(id: string): Tutorial | undefined {
    return ALL_TUTORIALS.find(tutorial => tutorial.id === id);
}
