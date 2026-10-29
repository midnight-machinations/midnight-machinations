export type VoicePeer = { id: number; name: string; playerIndex: number | null; inVoice: boolean };
export type VoiceSession = {
    url: string;
    token: string;
    scope: string;
    selfId: number;
    sendGroups: string[];
    receiveGroups: string[];
    forcedSilenced: boolean;
    peers: VoicePeer[];
};
export type GroupPreference = { muted: boolean; deafened: boolean };
export type VoiceSettings = {
    mode: "pushToTalk" | "openMic" | "toggleMute";
    key: string;
    deviceId: string;
    volume: number;
    noiseSuppression: boolean;
    thresholdDb: number;
};
