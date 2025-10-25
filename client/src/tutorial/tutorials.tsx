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
    serverBased: false,
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
 * Understanding Roles Tutorial
 * Teaches about different role categories and factions
 */
export const understandingRolesTutorial: Tutorial = {
    id: "understanding-roles",
    name: "Understanding Roles",
    description: "Learn about the different roles and factions in the game",
    order: 2,
    serverBased: false,
    initialSetup: {
        playerRole: "detective",
        playerCount: 9,
        startPhase: "briefing",
        startDay: 1
    },
    steps: [
        {
            id: "intro",
            title: "Understanding Roles",
            description: "In Midnight Machinations, each player has a unique role with special abilities. Understanding roles is key to playing effectively.",
            completionCondition: { type: "manual" }
        },
        {
            id: "town-faction",
            title: "Town Faction",
            description: "The Town is the largest faction and includes roles like Villager, Detective, Doctor, and many others. Town members work together to identify and eliminate evil players.",
            completionCondition: { type: "manual" }
        },
        {
            id: "mafia-faction",
            title: "Mafia Faction",
            description: "The Mafia is the main antagonist faction. Mafia members know each other and can communicate privately. They choose one person to attack each night.",
            completionCondition: { type: "manual" }
        },
        {
            id: "neutral-roles",
            title: "Neutral Roles",
            description: "Neutral roles have their own unique win conditions. Some neutrals are friendly to Town, while others are hostile. Examples include Jester, Survivor, and Executioner.",
            completionCondition: { type: "manual" }
        },
        {
            id: "investigative-roles",
            title: "Investigative Roles",
            description: "Investigative roles like Detective and Lookout can gather information about other players at night. This information helps the Town find evil players.",
            completionCondition: { type: "manual" }
        },
        {
            id: "protective-roles",
            title: "Protective Roles",
            description: "Protective roles like Doctor can prevent attacks on players at night. Keeping important Town members alive is crucial to winning.",
            completionCondition: { type: "manual" }
        },
        {
            id: "killing-roles",
            title: "Killing Roles",
            description: "Some Town roles like Vigilante can kill players at night. These powerful roles must be used carefully - killing a Town member by mistake can cost you the game!",
            completionCondition: { type: "manual" }
        },
        {
            id: "conclusion",
            title: "Role Knowledge is Power",
            description: "Understanding what each role can do helps you analyze claims, spot inconsistencies, and make better decisions. Check the Wiki to learn more about specific roles!",
            completionCondition: { type: "manual" }
        }
    ]
};

/**
 * Detective Tutorial (Interactive)
 * Interactive tutorial teaching how to play as Detective
 */
export const detectiveTutorial: Tutorial = {
    id: "detective-tutorial",
    name: "Playing as Detective",
    description: "Interactive tutorial: Learn how to investigate players and share your findings",
    order: 3,
    serverBased: true,
    initialSetup: {
        playerRole: "detective",
        playerCount: 9,
        startPhase: "briefing",
        startDay: 1
    },
    steps: [
        {
            id: "welcome",
            title: "Welcome, Detective!",
            description: "You are a Detective, one of the Town's most powerful investigative roles. Your job is to find evil players by investigating them at night. This is an interactive tutorial - you'll actually perform detective actions to learn.",
            completionCondition: { type: "manual" }
        },
        {
            id: "briefing-phase",
            title: "Briefing Phase",
            description: "The game begins with the Briefing phase where you learn your role. As Detective, you can investigate one player each night to determine if they're suspicious or innocent. Let's wait for the Night phase to use your ability.",
            completionCondition: { type: "manual" }
        },
        {
            id: "select-target",
            title: "Choose Someone to Investigate",
            description: "During the Night, open the Abilities menu and select a player to investigate. For this tutorial, investigate player #2 (Alice). Click on Alice in the abilities menu to select her as your target.",
            completionCondition: { 
                type: "action",
                actionType: { type: "selectTarget", controllerId: "detective-investigation" }
            }
        },
        {
            id: "obituary",
            title: "Check Your Results",
            description: "Great! During the Obituary phase (the start of each day), you'll receive the investigation result. Alice's result will appear in your chat. Read it carefully - it tells you whether she is innocent or suspicious.",
            completionCondition: { type: "manual" }
        },
        {
            id: "write-will",
            title: "Document Your Findings",
            description: "As Detective, you should keep track of your investigation results. Open the Alibi menu (📜) and write down that you investigated Alice. Include what you found. This will help others trust your information if you die.",
            completionCondition: { 
                type: "action",
                actionType: { type: "writeWill" }
            }
        },
        {
            id: "share-results",
            title: "Share Your Information",
            description: "Now that Day has started, you can share your findings in chat. Type a message telling everyone that you investigated Alice and what you found. Good detectives share their results to help Town find evil players!",
            completionCondition: { 
                type: "action",
                actionType: { type: "sendChat" }
            }
        },
        {
            id: "next-night",
            title: "Continue Investigating",
            description: "Excellent work! As Detective, you should investigate a different player every night. Over time, you'll build up a list of innocent and suspicious players. This information is crucial for Town to win.",
            completionCondition: { type: "manual" }
        },
        {
            id: "tips",
            title: "Detective Tips",
            description: "Key tips: (1) Always write your results in your alibi, (2) Share results in chat during the day, (3) Investigate suspicious players or claims, (4) If you find evil players, push to get them voted out!",
            completionCondition: { type: "manual" }
        },
        {
            id: "conclusion",
            title: "Tutorial Complete!",
            description: "You now know how to play as Detective! Remember: investigate, document, and share your findings. Your information can turn the tide of the game. Good luck hunting down the Mafia!",
            completionCondition: { type: "manual" }
        }
    ]
};

/**
 * Array of all available tutorials
 */
export const ALL_TUTORIALS: Tutorial[] = [
    basicGameplayTutorial,
    understandingRolesTutorial,
    detectiveTutorial
];

/**
 * Get a tutorial by ID
 */
export function getTutorialById(id: string): Tutorial | undefined {
    return ALL_TUTORIALS.find(tutorial => tutorial.id === id);
}
