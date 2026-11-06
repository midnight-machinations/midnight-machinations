import { ChatMessage, ChatMessageVariant, encodeString, translateChatMessage } from "../components/ChatMessage";
import { PlayerIndex, UnsafeString } from "./gameState.d";
import { translateGraveDeathCauses } from "./graveState";
import translate from "./lang";
import { RoleList } from "./roleListState.d";

export type ChatFilter = {
    type: "text";
    text: string;
} | {
    type: "playerNameInMessage";
    player: PlayerIndex;
} | {
    type: "alibiBlocks" | "playerMessages" | "nonPlayer" | "nightResults";
} | {
    type: "whispersBetweenPlayers";
    players: PlayerIndex[];
} | null;

function searchableText(message: ChatMessageVariant, names: UnsafeString[], roles: RoleList): string {
    switch (message.type) {
        case "normal": {
            const sender = message.messageSender;
            const name = sender.type === "player" || sender.type === "livingToDead"
                ? encodeString(names[sender.player] ?? "") : translate(`role.${sender.type}.name`);
            return `${name} ${translateChatMessage(message, names, roles)}`;
        }
        case "targetsMessage":
        case "playerForwardedMessage":
            return `${translateChatMessage(message, names, roles)} ${searchableText(message.message, names, roles)}`;
        case "playerDied": {
            const { grave } = message;
            const info = grave.information;
            return `${encodeString(names[grave.player] ?? "")} ${info.type === "normal"
                ? `${translate(`role.${info.role}.name`)} ${encodeString(info.alibi)} ${translateGraveDeathCauses(info.deathCauses)} ${info.callingCards.map(encodeString).join(" ")}`
                : translate("obscured")}`;
        }
        case "kiraResult":
            return `${translate("chatMessage.kiraResult")} ${message.result.guesses.map(([player, [guess, result]]) =>
                `${encodeString(names[player] ?? "")} ${translate(guess === "none" || guess === "nonTown" ? guess : `role.${guess}.name`)} ${translate(`kiraResult.${result}`)}`
            ).join(" ")}`;
        default:
            return translateChatMessage(message, names, roles);
    }
}

export function filterChatMessage(filter: ChatFilter, message: ChatMessage, names: UnsafeString[], roles: RoleList, plainNames: UnsafeString[] = names): boolean {
    if (filter === null) return true;
    const variant = message.variant;
    switch (filter.type) {
        case "text":
            return searchableText(variant, names, roles).toLocaleLowerCase().includes(encodeString(filter.text.trim()).toLocaleLowerCase());
        case "playerNameInMessage": {
            const name = plainNames[filter.player];
            return name !== undefined && searchableText(variant, names, roles).toLocaleLowerCase().includes(encodeString(name).toLocaleLowerCase());
        }
        case "alibiBlocks":
            return variant.type === "normal" && variant.block;
        case "playerMessages":
            return isPlayerMessage(variant);
        case "nonPlayer":
            return !isPlayerMessage(variant);
        case "nightResults":
            return message.nightResult === true;
        case "whispersBetweenPlayers":
            return variant.type === "whisper" && filter.players.includes(variant.fromPlayerIndex) && filter.players.includes(variant.toPlayerIndex);
    }
}

function isPlayerMessage(message: ChatMessageVariant): boolean {
    return ["normal", "whisper", "broadcastWhisper", "lobbyMessage", "playerForwardedMessage", "reporterReport"].includes(message.type);
}

export function filterChatMessages<T extends readonly [unknown, ChatMessage]>(
    filter: ChatFilter, messages: readonly T[], names: UnsafeString[], roles: RoleList, plainNames: UnsafeString[] = names
): T[] {
    if (filter === null) return [...messages];
    const visible = messages.map(([, message]) => filterChatMessage(filter, message, names, roles, plainNames));
    let phaseIndex: number | undefined;
    messages.forEach(([, message], index) => {
        if (message.variant.type === "phaseChange") phaseIndex = index;
        else if (visible[index] && phaseIndex !== undefined) visible[phaseIndex] = true;
    });
    return messages.filter((_, index) => visible[index]);
}
