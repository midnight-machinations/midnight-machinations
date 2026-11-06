/**
 * Minimal client for the Cloudflare Realtime SFU HTTP API.
 * https://developers.cloudflare.com/realtime/sfu/https-api/
 *
 * The app token never leaves the worker, so browsers can only receive the
 * tracks this service pulls into their session.
 */
export type SessionDescription = { type: "offer" | "answer"; sdp: string };
export type TrackResult = { mid?: string; trackName?: string; sessionId?: string; errorCode?: string; errorDescription?: string };
type TracksResponse = { sessionDescription?: SessionDescription; requiresImmediateRenegotiation?: boolean; tracks?: TrackResult[] };

export class Sfu {
    constructor(private readonly appId: string, private readonly token: string) {}

    private async call<T>(method: string, path: string, body?: unknown): Promise<T> {
        const response = await fetch(`https://rtc.live.cloudflare.com/v1/apps/${this.appId}${path}`, {
            method,
            headers: { authorization: `Bearer ${this.token}`, "content-type": "application/json" },
            body: body === undefined ? undefined : JSON.stringify(body),
        });
        type Result = T & { errorCode?: string; errorDescription?: string };
        const result = await response.json<Result>().catch(() => ({}) as Result);
        if (!response.ok || result.errorCode) {
            throw new Error(`SFU ${method} ${path.replace(/[0-9a-f]{32,}/g, ":id")} failed: ${result.errorDescription ?? response.status}`);
        }
        return result;
    }

    async newSession(): Promise<string> {
        return (await this.call<{ sessionId: string }>("POST", "/sessions/new")).sessionId;
    }

    /** Pushes the browser's microphone transceiver as the session's single track. */
    publish(session: string, offer: SessionDescription, mid: string): Promise<TracksResponse> {
        return this.call("POST", `/sessions/${session}/tracks/new`, {
            sessionDescription: offer,
            tracks: [{ location: "local", mid, trackName: "voice" }],
        });
    }

    /** Pulls each speaker session's track into `session`. Usually requires renegotiation. */
    pull(session: string, speakers: string[]): Promise<TracksResponse> {
        return this.call("POST", `/sessions/${session}/tracks/new`, {
            tracks: speakers.map(sessionId => ({ location: "remote", sessionId, trackName: "voice" })),
        });
    }

    /**
     * Stops forwarding without renegotiating, so revocation never waits for the
     * browser and the listener gets no signal beyond silence.
     */
    close(session: string, mids: string[]): Promise<TracksResponse> {
        return this.call("PUT", `/sessions/${session}/tracks/close`, { tracks: mids.map(mid => ({ mid })), force: true });
    }

    renegotiate(session: string, answer: SessionDescription): Promise<unknown> {
        return this.call("PUT", `/sessions/${session}/renegotiate`, { sessionDescription: answer });
    }
}
