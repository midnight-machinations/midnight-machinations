// biome-ignore-all lint/suspicious/noExplicitAny: these tests run compiled sources in a vm sandbox with fake modules
import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";
import { readSource, withoutImports } from "./support.ts";

function setup() {
    const cards: any[] = [];
    const context: any = {
        console: { warn() {} },
        window: { addEventListener() {}, setInterval() {} },
        document: { addEventListener() {} },
        localStorage: { getItem: () => null, setItem() {} },
        ANCHOR_CONTROLLER: { pushErrorCard: (card: unknown) => cards.push(card) },
        translate: (key: string) => key,
    };
    vm.createContext(context);
    const source = withoutImports(readSource("voice/voiceManager.ts"))
        .replace("export class VoiceManager", "class VoiceManager")
        .replace("export const VOICE = new VoiceManager();", "globalThis.VoiceManager = VoiceManager;");
    vm.runInContext(ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText, context);
    const voice = new context.VoiceManager();
    const audio = { state: "running", currentTime: 0 };
    voice.audio = audio;
    voice.wantConnected = true;
    return { voice, audio, cards };
}

function check(voice: any, times: number, { advance = false } = {}) {
    for (let i = 0; i < times; i++) {
        if (advance) voice.audio.currentTime += 1;
        voice.checkAudioRunning();
    }
}

test("audio whose clock keeps moving shows no warning", () => {
    const { voice, cards } = setup();
    check(voice, 10, { advance: true });
    assert.equal(voice.error, "");
    assert.equal(cards.length, 0);
});

test("a clock that stops while the context says running shows the stuck warning once", () => {
    const { voice, cards } = setup();
    check(voice, 2, { advance: true });
    check(voice, 10);
    assert.equal(voice.error, "voice.error.stuckAudio");
    assert.equal(cards.length, 1);
    assert.equal(cards[0].body, "voice.error.stuckAudio");
});

test("a context that never starts shows the stuck warning", () => {
    const { voice, audio } = setup();
    audio.state = "suspended";
    check(voice, 3);
    assert.equal(voice.error, "voice.error.stuckAudio");
});

test("a short pause shows no warning", () => {
    const { voice, audio } = setup();
    check(voice, 1, { advance: true });
    audio.state = "suspended";
    check(voice, 2);
    audio.state = "running";
    check(voice, 5, { advance: true });
    assert.equal(voice.error, "");
});

test("the warning clears if audio starts working again", () => {
    const { voice } = setup();
    check(voice, 4);
    assert.equal(voice.error, "voice.error.stuckAudio");
    check(voice, 2, { advance: true });
    assert.equal(voice.error, "");
});

test("nothing is checked while voice is off", () => {
    const { voice } = setup();
    voice.wantConnected = false;
    check(voice, 10);
    assert.equal(voice.error, "");
});

test("the generic silent-microphone warning does not replace the stuck warning", async () => {
    const { voice, cards } = setup();
    check(voice, 5);
    assert.equal(voice.error, "voice.error.stuckAudio");
    const sent = 0;
    voice.sender = { track: {}, getStats: async () => new Map([["out", { type: "outbound-rtp", packetsSent: sent }]]) };
    voice.status = "connected";
    voice.microphoneEnabled = true;
    await voice.checkMicrophoneFlow();
    await voice.checkMicrophoneFlow();
    assert.equal(voice.error, "voice.error.stuckAudio");
    assert.equal(cards.length, 1);
});

test("without stuck audio, stalled microphone packets still show the silent-microphone warning", async () => {
    const { voice } = setup();
    voice.sender = { track: {}, getStats: async () => new Map([["out", { type: "outbound-rtp", packetsSent: 5 }]]) };
    voice.status = "connected";
    voice.microphoneEnabled = true;
    await voice.checkMicrophoneFlow();
    await voice.checkMicrophoneFlow();
    assert.equal(voice.error, "voice.error.silentMicrophone");
});
