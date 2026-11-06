import assert from "node:assert/strict";
import { test } from "node:test";
import { canForwardVoice, type Policy } from "../src/routing.ts";

const member = (id: number, send: string[], receive: string[], scope = "room"): Policy => ({ id, token: `t${id}`, scope, send, receive });

test("forwards a speaker's one send group to listeners who receive it", () => {
    assert.equal(canForwardVoice(member(1, ["all"], []), member(2, [], ["all"]), new Set()), true);
    assert.equal(canForwardVoice(member(1, ["mafia"], []), member(2, [], ["all"]), new Set()), false);
});

test("refuses multiple send groups, other scopes, deafened groups, and self", () => {
    const listener = member(2, [], ["all", "mafia"]);
    assert.equal(canForwardVoice(member(1, ["all", "mafia"], []), listener, new Set()), false);
    assert.equal(canForwardVoice(member(1, ["all"], [], "other"), listener, new Set()), false);
    assert.equal(canForwardVoice(member(1, ["all"], []), listener, new Set(["all"])), false);
    assert.equal(canForwardVoice(listener, listener, new Set()), false);
    assert.equal(canForwardVoice(undefined, listener, new Set()), false);
});
