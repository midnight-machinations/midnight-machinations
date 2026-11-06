// biome-ignore-all lint/suspicious/noExplicitAny: these tests run compiled sources in a vm sandbox with fake modules
import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";
import { readSource, withoutImports } from "./support.ts";

const voice = {
    status: "disconnected",
    roomEnabled: true,
    error: "",
    settings: { mode: "openMic" },
    microphoneEnabled: false,
    session: {
        selfId: 1,
        sendGroups: ["all"],
        receiveGroups: ["all"],
        forcedSilenced: false,
        peers: [
            { id: 1, name: "Me" },
            { id: 2, name: "Alice" },
            { id: 3, name: "<Bob>" },
        ],
    },
    group: () => ({ deafened: false }),
    person: () => ({ volume: 1 }),
};
const context: any = {
    React: { createElement: (type: unknown, props: unknown, ...children: unknown[]) => ({ type, props, children }) },
    VOICE: voice,
    useSyncExternalStore() {},
    translate: (key: string) => (key === "voice.groups" ? "Deafen by group" : key),
    encodeString: (value: string) => value.replaceAll("<", "&lt;").replaceAll(">", "&gt;"),
    StyledText: function StyledText() {},
    SpeakingIndicator: function SpeakingIndicator() {},
    Icon: function Icon() {},
};
const source = withoutImports(readSource("voice/VoiceControls.tsx"))
    .replace(/export default /g, "")
    .replace(/export /g, "");
vm.createContext(context);
vm.runInContext(ts.transpileModule(source, { compilerOptions: { jsx: ts.JsxEmit.React } }).outputText, context);
function find(node: any, predicate: (node: any) => boolean): any[] {
    if (!node) return [];
    if (Array.isArray(node)) return node.flatMap((child) => find(child, predicate));
    return [...(predicate(node) ? [node] : []), ...find(node.children, predicate)];
}

for (const status of ["disconnected", "connecting", "error"]) {
    test(`individual volumes are absent while ${status}`, () => {
        voice.status = status;
        const tree = context.VoiceControls();
        assert.equal(find(tree, (node) => node.type === "details").length, 0);
        assert.equal(find(tree, (node) => node.type === "input" && node.props?.type === "range").length, 0);
    });
}

test("connected voice shows volume controls for other players only", () => {
    voice.status = "connected";
    const tree = context.VoiceControls();
    assert.equal(find(tree, (node) => node.type === "details").length, 1);
    assert.equal(find(tree, (node) => node.type === "input" && node.props?.type === "range").length, 2);
    assert.equal(
        find(tree, (node) => node.type === context.StyledText && node.children[0] === "&lt;Bob&gt;").length,
        1,
    );
});

test("group deafen label uses StyledText", () => {
    assert.equal(
        find(
            context.VoiceControls(),
            (node) => node.type === context.StyledText && node.children[0] === "Deafen by group",
        ).length,
        1,
    );
});
