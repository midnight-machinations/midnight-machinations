import { test } from 'node:test';
import assert from 'node:assert/strict';
import { canForwardVoice } from '../src/routing.mjs';

const policy = (send = [], receive = [], scope = 'game') => ({ send, receive, scope });

test('voice reaches only listeners of the selected group', () => {
  const speaker = policy(['mafia']);
  assert.equal(canForwardVoice(speaker, policy([], ['mafia']), new Set()), true);
  assert.equal(canForwardVoice(speaker, policy([], ['all']), new Set()), false);
  assert.equal(canForwardVoice(speaker, policy([], ['all', 'mafia']), new Set()), true);
});

test('switching the selected group revokes the old route', () => {
  const listener = policy([], ['mafia']);
  assert.equal(canForwardVoice(policy(['mafia']), listener, new Set()), true);
  assert.equal(canForwardVoice(policy(['all']), listener, new Set()), false);
});

test('group deafen still blocks the selected group', () => {
  assert.equal(canForwardVoice(policy(['mafia']), policy([], ['mafia']), new Set(['mafia'])), false);
  assert.equal(canForwardVoice(policy(['mafia']), policy([], ['mafia']), new Set(['all'])), true);
});

test('missing permissions, different rooms, and multiple send groups fail closed', () => {
  const listener = policy([], ['all', 'mafia']);
  assert.equal(canForwardVoice(undefined, listener, new Set()), false);
  assert.equal(canForwardVoice(policy(['all']), undefined, new Set()), false);
  assert.equal(canForwardVoice(policy(['all'], [], 'other room'), listener, new Set()), false);
  assert.equal(canForwardVoice(policy([]), listener, new Set()), false);
  assert.equal(canForwardVoice(policy(['all', 'mafia']), listener, new Set()), false);
});

test('lobby voice retains its single shared group', () => {
  assert.equal(canForwardVoice(policy(['lobby'], ['lobby'], 'lobby'), policy(['lobby'], ['lobby'], 'lobby'), new Set()), true);
});
