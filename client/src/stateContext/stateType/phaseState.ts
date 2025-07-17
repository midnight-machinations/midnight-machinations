import { PlayerIndex } from "./otherState";

export const PHASES = ["briefing", "obituary", "discussion", "nomination", "testimony", "judgement", "finalWords", "dusk", "night", "recess"] as const;
export type PhaseType = (typeof PHASES)[number];
export type PhaseState = {type: "briefing"} | {type: "recess"} | {type: "dusk"} | {type: "night"} | {type: "obituary"} | {type: "discussion"} | 
{
    type: "nomination",
    trialsLeft: number
} | {
    type: "testimony",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "judgement",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "finalWords",
    playerOnTrial: PlayerIndex
}