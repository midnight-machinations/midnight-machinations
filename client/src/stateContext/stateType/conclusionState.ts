import translate from "../../game/lang"

export type WinCondition = {
    type: "gameConclusionReached"
    winIfAny: Conclusion[]
} | {
    type: "roleStateWon"
}
export function translateWinCondition(winCondition: WinCondition): string {
    if (winCondition.type === "gameConclusionReached") {
        if (winCondition.winIfAny.length === 0) {
            return translate("winCondition.loser")
        } else if (winCondition.winIfAny.length === 1) {
            return translateConclusion(winCondition.winIfAny[0])
        } else if (winCondition.winIfAny.length === 4 && 
            (["mafia", "fiends", "cult", "politician"] as const).every(team => winCondition.winIfAny.includes(team))
        ) {
            return translate(`winCondition.evil`)
        } else {
            return winCondition.winIfAny.map(conclusion => translateConclusion(conclusion)).join(` ${translate('union')} `)
        }
    } else {
        return translate("winCondition.independent");
    }
}

export const CONCLUSIONS = ["town", "mafia", "cult", "fiends", "politician", "niceList", "naughtyList", "draw"] as const;
export type Conclusion = (typeof CONCLUSIONS)[number];
export function translateConclusion(conclusion: Conclusion): string {
    switch (conclusion) {
        case "politician":
            return translate("role.politician.name")
        case "draw":
            return translate("winCondition.draw")
        default:
            return translate(conclusion)
    }
}