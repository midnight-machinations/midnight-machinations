// biome-ignore-all lint/suspicious/noExplicitAny: these tests run compiled sources in a vm sandbox with fake modules
import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";
import { readSource, withoutImports } from "./support.ts";

const encode = (text: unknown) => String(text).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
const names = ["1: Alice", "2: Bob"];
const plain = ["Alice", "Bob"];
function translateMessage(message: any, names: string[]) {
    if (message.type === "phaseChange") return `${message.phase.type} ${message.dayNumber ?? ""} ${names[message.phase.playerOnTrial] ?? ""}`;
    if (message.type === "whisper") return `${names[message.fromPlayerIndex]} to ${names[message.toPlayerIndex]}: ${encode(message.text)}`;
    if (message.type === "playerForwardedMessage") return `${names[message.forwarder]} forwarded`;
    return encode(message.text ?? message.report ?? message.type);
}
const exportsFilter: any = {};
vm.runInNewContext(ts.transpileModule(readSource("game/chatFilter.ts"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS },
}).outputText, {
    exports: exportsFilter,
    require: (name: string) => {
        if (name === "../components/ChatMessage") return { encodeString: encode, translateChatMessage: translateMessage };
        if (name === "./lang") return { default: (key: string) => key };
        if (name === "./graveState") return { translateGraveDeathCauses: () => "execution" };
        throw new Error(name);
    },
});
const filterMessage = (filter: unknown, variant: unknown, extra = {}) => exportsFilter.filterChatMessage(filter, { variant, chatGroup: "all", ...extra }, names, [], plain);
const normal = (text: string, block = false, player = 0) => ({ type: "normal", text, block, messageSender: { type: "player", player } });

test("phase headers can match a search directly, without automatically matching other filters", () => {
    const phase = { type: "phaseChange", phase: { type: "night" }, dayNumber: 2 };
    assert.equal(filterMessage(null, phase), true);
    assert.equal(filterMessage({ type: "text", text: "night" }, phase), true);
    assert.equal(filterMessage({ type: "text", text: "unrelated" }, phase), false);
    assert.equal(filterMessage({ type: "nonPlayer" }, phase), true);
    for (const type of ["alibiBlocks", "playerMessages", "nightResults"]) assert.equal(filterMessage({ type }, phase), false);
});

test("text search is literal and case-insensitive, with empty search showing all messages", () => {
    assert.equal(filterMessage({ type: "text", text: "hello" }, normal("HELLO everyone")), true);
    assert.equal(filterMessage({ type: "text", text: "[test]" }, normal("[TEST]")), true);
    assert.equal(filterMessage({ type: "text", text: "<hi>" }, normal("<HI>")), true);
    assert.equal(filterMessage({ type: "text", text: "bob" }, normal("hello")), false);
    assert.equal(filterMessage({ type: "text", text: "   " }, normal("hello")), true);
});

test("player-name search matches authors and mentions without requiring the player number", () => {
    const filter = { type: "playerNameInMessage", player: 0 };
    assert.equal(filterMessage(filter, normal("hello")), true);
    assert.equal(filterMessage(filter, normal("I trust Alice", false, 1)), true);
    assert.equal(filterMessage(filter, normal("I trust nobody", false, 1)), false);
    assert.equal(filterMessage({ type: "playerNameInMessage", player: 99 }, normal("hello")), false);
});

test("search descends into forwarded and target-result messages", () => {
    const forwarded = { type: "playerForwardedMessage", forwarder: 1, message: normal("secret clue") };
    const targeted = { type: "targetsMessage", message: normal("secret clue") };
    assert.equal(filterMessage({ type: "text", text: "CLUE" }, forwarded), true);
    assert.equal(filterMessage({ type: "text", text: "CLUE" }, targeted), true);
});

test("alibi filter includes only player block messages", () => {
    assert.equal(filterMessage({ type: "alibiBlocks" }, normal("alibi", true)), true);
    assert.equal(filterMessage({ type: "alibiBlocks" }, normal("chat")), false);
    assert.equal(filterMessage({ type: "alibiBlocks" }, { type: "reporterReport", report: "report" }), false);
});

test("non-player filter excludes player text, blocks, whispers, reports, and forwards", () => {
    const filter = { type: "nonPlayer" };
    for (const message of [normal("chat"), normal("alibi", true), { ...normal("anonymous"), messageSender: { type: "jailor" } }, { type: "whisper" }, { type: "broadcastWhisper" }, { type: "reporterReport" }, { type: "playerForwardedMessage" }, { type: "lobbyMessage" }]) {
        assert.equal(filterMessage(filter, message), false);
    }
    assert.equal(filterMessage(filter, { type: "roleBlocked" }), true);
    assert.equal(filterMessage(filter, { type: "trialVerdict" }), true);
});

test("night-result filtering uses authoritative delivery metadata, including warnings", () => {
    assert.equal(filterMessage({ type: "nightResults" }, { type: "silenced" }, { nightResult: true }), true);
    assert.equal(filterMessage({ type: "nightResults" }, { type: "detectiveResult" }, { nightResult: true }), true);
    assert.equal(filterMessage({ type: "nightResults" }, { type: "detectiveResult" }), false);
    assert.equal(filterMessage({ type: "nightResults" }, normal("chat")), false);
});

test("grave and Kira message searches use their displayed information", () => {
    const grave = { type: "playerDied", grave: { player: 1, information: { type: "normal", role: "detective", alibi: "My will", deathCauses: [], callingCards: ["a clue"] } } };
    assert.equal(filterMessage({ type: "text", text: "my will" }, grave), true);
    assert.equal(filterMessage({ type: "playerNameInMessage", player: 0 }, grave), false);
    assert.equal(filterMessage({ type: "playerNameInMessage", player: 1 }, grave), true);
    const kira = { type: "kiraResult", result: { guesses: [[0, ["detective", "correct"]]] } };
    assert.equal(filterMessage({ type: "text", text: "Alice" }, kira), true);
});

test("existing whisper conversations remain scoped to their participants", () => {
    const filter = { type: "whispersBetweenPlayers", players: [0, 1] };
    assert.equal(filterMessage(filter, { type: "whisper", fromPlayerIndex: 0, toPlayerIndex: 1, text: "hi" }), true);
    assert.equal(filterMessage(filter, { type: "whisper", fromPlayerIndex: 0, toPlayerIndex: 2, text: "hi" }), false);
    assert.equal(filterMessage(filter, normal("hi")), false);
});

const react = { createElement: (type: unknown, props: unknown, ...children: unknown[]) => ({ type, props, children }) };
const sent: any[] = [];
const controls: any = {};
vm.runInNewContext(ts.transpileModule(readSource("menu/game/gameScreenContent/ChatFilters.tsx"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.React, esModuleInterop: true },
}).outputText, {
    exports: controls,
    require: (name: string) => {
        if (name === "react") return react;
        if (name === "../../..") return { updateChatFilter: (filter: unknown) => sent.push(filter) };
        if (name.endsWith("/lang")) return (key: string) => key;
        if (name.endsWith("/useHooks")) return { usePlayerNames: () => names };
        if (name.endsWith("/Button")) return { Button: function Button() {} };
        if (name.endsWith("/Icon")) return function Icon() {};
        if (name.endsWith("/StyledText")) return function StyledText() {};
        if (name.endsWith("/ChatMessage")) return { encodeString: encode };
        if (name.endsWith("/PlayerOptionDropdown")) return function PlayerOptionDropdown() {};
        if (name.endsWith("/Select")) return function Select() {};
        throw new Error(name);
    },
});

test("default filter control is an icon and its menu contains only actual filters", () => {
    const view = controls.default({ filter: null });
    const props = view.children[0].children[1].props;
    assert.equal(props.value, null);
    assert.equal(props.placeholder.children[0], "filter_alt");
    assert.equal(props.hideArrow, true);
    assert.deepEqual([...props.optionsSearch.keys()], ["text", "playerNameInMessage", "playerMessages", "alibiBlocks", "nonPlayer", "nightResults"]);
    for (const mode of ["playerMessages", "alibiBlocks", "nonPlayer", "nightResults"]) {
        props.onChange(mode);
        assert.equal(sent.at(-1).type, mode);
    }
    assert.equal(props.optionsSearch.has("all"), false);
});

test("text and player controls update the chosen filter and can be cleared", () => {
    let view = controls.default({ filter: { type: "text", text: "old" } });
    assert.equal(view.children[1].props.value, "old");
    view.children[1].props.onChange({ target: { value: "new text" } });
    assert.equal(sent.at(-1).text, "new text");
    view = controls.default({ filter: { type: "playerNameInMessage", player: 0 } });
    view.children[2].props.onChange(1);
    assert.equal(sent.at(-1).player, 1);
    view.children[0].children[0].props.onClick();
    assert.equal(sent.at(-1), null);
});

test("chat list hides empty phases during search and restores them when cleared", () => {
    const phaseMessages = [
        [0, { variant: { type: "phaseChange", phase: { type: "night" } }, chatGroup: "all" }],
        [1, { variant: { type: "phaseChange", phase: { type: "obituary" } }, chatGroup: "all" }],
        [2, { variant: { type: "phaseChange", phase: { type: "discussion" } }, chatGroup: "all" }],
    ];
    const game = { players: [], roleList: [], chatMessages: { entries: () => phaseMessages } };
    const source = withoutImports(readSource("menu/game/gameScreenContent/ChatMenu.tsx"))
        .replace(/export default /g, "").replace(/export /g, "");
    const context: any = {
        React: react, useMemo: (fn: () => unknown) => fn(), usePlayerNames: () => names,
        useGameState: (fn: (game: unknown) => unknown) => fn(game), useLobbyOrGameState: (fn: (game: unknown) => unknown) => fn(game),
        filterChatMessages: exportsFilter.filterChatMessages,
        ChatElement: function ChatElement() {}, Virtuoso: function Virtuoso() {},
        ROLE_LIST_KEYWORD_DATA: {},
    };
    vm.createContext(context);
    vm.runInContext(ts.transpileModule(source, { compilerOptions: { jsx: ts.JsxEmit.React } }).outputText, context);
    const view = context.ChatMessageSection({ filter: { type: "text", text: "no matches" } });
    assert.equal(view.children[0].props.totalCount, 0);
    assert.equal(context.ChatMessageSection({ filter: null }).children[0].props.totalCount, 3);
    assert.equal(context.ChatMessageSection({ filter: { type: "text", text: "night" } }).children[0].props.totalCount, 1);
});

test("player messages include whispers, broadcasts, forwarded messages, and Reporter blocks", () => {
    const filter = { type: "playerMessages" };
    for (const variant of [normal("chat"), normal("block", true), { type: "whisper" }, { type: "broadcastWhisper" }, { type: "playerForwardedMessage" }, { type: "reporterReport" }, { type: "lobbyMessage" }]) {
        assert.equal(filterMessage(filter, variant), true);
        assert.equal(filterMessage({ type: "nonPlayer" }, variant), false);
    }
    assert.equal(filterMessage(filter, { type: "roleBlocked" }), false);
});

const entry = (index: number, variant: unknown) => [index, { variant, chatGroup: "all" }];
const phase = (name: string, day = 2) => ({ type: "phaseChange", phase: { type: name }, dayNumber: day });
const filteredIds = (filter: unknown, messages: unknown[]) => Array.from(exportsFilter.filterChatMessages(filter, messages, names, [], plain), ([id]: any) => id);

test("matching messages retain their own phase header but not empty neighboring phases", () => {
    const messages = [
        entry(0, phase("night")), entry(1, normal("hello")),
        entry(2, phase("obituary")), entry(3, { type: "roleBlocked" }),
        entry(4, phase("discussion")), entry(5, normal("HELLO")),
        entry(6, phase("night", 3)),
    ];
    assert.deepEqual(filteredIds({ type: "text", text: "hello" }, messages), [0, 1, 4, 5]);
    assert.deepEqual(filteredIds({ type: "playerMessages" }, messages), [0, 1, 4, 5]);
    assert.deepEqual(filteredIds(null, messages), [0, 1, 2, 3, 4, 5, 6]);
});

test("a phase header that matches directly stays visible without matching messages beneath it", () => {
    const messages = [entry(0, phase("night")), entry(1, phase("obituary")), entry(2, phase("discussion"))];
    assert.deepEqual(filteredIds({ type: "text", text: "night" }, messages), [0]);
    assert.deepEqual(filteredIds({ type: "nonPlayer" }, messages), [0, 1, 2]);
    assert.deepEqual(filteredIds({ type: "nightResults" }, messages), []);
});

test("phase context does not reveal nonmatching messages and handles partial history", () => {
    const messages = [entry(0, normal("alibi", true)), entry(1, phase("night")), entry(2, normal("ordinary")), entry(3, normal("alibi", true))];
    assert.deepEqual(filteredIds({ type: "alibiBlocks" }, messages), [0, 1, 3]);
});

test("Select renders an icon placeholder without adding it to the dropdown choices", () => {
    const exportsSelect: any = {};
    const hookReact = {
        ...react, useState: (value: unknown) => [value, () => {}], useRef: (value: unknown) => ({ current: value }),
        useMemo: (fn: () => unknown) => fn(), useCallback: (fn: () => unknown) => fn(), useEffect() {},
    };
    vm.runInNewContext(ts.transpileModule(readSource("components/Select.tsx"), {
        compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.React, esModuleInterop: true },
    }).outputText, {
        exports: exportsSelect,
        require: (name: string) => {
            if (name === "react") return hookReact;
            if (name === "./Button") return { RawButton: function RawButton() {} };
            if (name.endsWith(".css")) return {};
            if (name === "./Popover") return function Popover() {};
            if (name === "./Icon") return function Icon() {};
            if (name === "../game/lang") return (key: string) => key;
            throw new Error(name);
        },
    });
    const choices = new Map([["text", ["Text search", "Text search"]]]);
    const placeholder = { icon: "filter_alt" };
    const view = exportsSelect.default({ value: null, optionsSearch: choices, placeholder, hideArrow: true, ariaLabel: "Filter messages" });
    assert.equal(view.children[0].children[1], placeholder);
    assert.equal(view.children[0].props["aria-label"], "Filter messages");
    assert.deepEqual([...choices.keys()], ["text"]);
});
