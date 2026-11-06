// biome-ignore-all lint/suspicious/noExplicitAny: these tests run compiled sources in a vm sandbox with fake modules
import assert from "node:assert/strict";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";
import { readSource } from "./support.ts";

const sent: any[] = [];
const moduleExports: any = {};
const compiled = ts.transpileModule(readSource("components/ChatGroupSelect.tsx"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.React, esModuleInterop: true },
}).outputText;
vm.runInNewContext(compiled, {
    exports: moduleExports,
    require: (name: string) => {
        if (name === "react") return { createElement: (type: unknown, props: unknown, ...children: unknown[]) => ({ type, props, children }) };
        if (name === "..") return { sendControllerInput: (input: unknown) => sent.push(input) };
        if (name === "../game/lang") return (key: string) => key;
        if (name === "./Select") return function Select() {};
        if (name === "./StyledText") return function StyledText() {};
        throw new Error(`Unexpected dependency: ${name}`);
    },
});
function render(groups: string[], selected: string | null, disabled = false, player = 0) {
    return moduleExports.default({ id: { type: "chatGroup", player }, groups, selected, disabled }).children[0].props;
}

test("keeps the current group when it remains allowed", () => {
    assert.equal(render(["all", "mafia"], "mafia").value, "mafia");
});

test("fallback prefers All, then the first allowed group", () => {
    assert.equal(render(["mafia", "all"], "dead").value, "all");
    assert.equal(render(["dead", "mafia"], "all").value, "dead");
});

test("only allowed groups are listed", () => {
    assert.deepEqual([...render(["all", "mafia"], "all").optionsSearch.keys()], ["all", "mafia"]);
});

test("choosing a group sends its validated controller input for the displayed player", () => {
    sent.length = 0;
    render(["dead", "mafia"], "dead", false, 3).onChange("mafia");
    assert.equal(sent.length, 1);
    assert.equal(sent[0].id.type, "chatGroup");
    assert.equal(sent[0].id.player, 3);
    assert.equal(sent[0].selection.type, "chatGroup");
    assert.equal(sent[0].selection.selection, "mafia");
});

test("no groups disables selection and shows None", () => {
    const props = render([], null);
    assert.equal(props.value, "none");
    assert.equal(props.disabled, true);
    sent.length = 0;
    props.onChange("none");
    assert.equal(sent.length, 0);
    assert.equal(render(["all"], "all", true).disabled, true);
});
