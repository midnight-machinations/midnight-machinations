/** A member's permissions as published by the game server. */
export type Policy = {
    id: number;
    token: string;
    scope: string;
    /** At most one group: the chat group this member is currently speaking to. */
    send: string[];
    receive: string[];
};

// The game server grants exactly one selected send group. Never forward a
// speaker into multiple groups, even if a malformed policy is received.
export function canForwardVoice(from: Policy | undefined, to: Policy | undefined, deafened: ReadonlySet<string>): boolean {
    if (!from || !to || from === to || from.scope !== to.scope || from.send.length !== 1) return false;
    const group = from.send[0];
    return to.receive.includes(group) && !deafened.has(group);
}
