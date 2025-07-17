import { PhaseType } from "./phaseState";

export type ClientConnection = "connected" | "disconnected" | "couldReconnect";

export type LobbyClientType = {
    type: "spectator"
} | PlayerClientType;
export type PlayerClientType = {
    type: "player",
    name: string,
}


export const INSIDER_GROUPS = ["mafia", "cult", "puppeteer"] as const;
export type InsiderGroup = (typeof INSIDER_GROUPS)[number];

export type PhaseTimes = Record<Exclude<PhaseType, "recess">, number>;
export function defaultPhaseTimes(): PhaseTimes {
    return {
        briefing: 45,
        obituary: 60,
        discussion: 120,
        nomination: 120,
        testimony: 30,
        judgement: 60,
        finalWords: 30,
        dusk: 30,
        night: 60,
    }
}


export type PlayerIndex = number;
export type LobbyClientID = number;
export type Verdict = "innocent"|"guilty"|"abstain";
export type ChatGroup = "all" | "dead" | "mafia" | "cult" | "jail" | "kidnapper" | "interview" | "puppeteer";
export type DefensePower = "none"|"armored"|"protected"|"invincible";



