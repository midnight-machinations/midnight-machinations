const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');

const keybind = {};
vm.runInNewContext(ts.transpileModule(fs.readFileSync(path.join(__dirname, '../src/voice/keybind.ts'), 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText, { exports: keybind });
const key = (code, modifiers = {}) => ({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...modifiers });

function setup(mode, binding) {
    const listeners = new Map();
    const saved = new Map([['voiceSettings', JSON.stringify({ mode, key: binding })]]);
    class Element { closest() { return this.editable ? this : null; } }
    const context = {
        ...keybind,
        window: { addEventListener: (type, handler) => listeners.set(type, handler), setInterval() {} },
        document: { addEventListener() {} }, Element,
        localStorage: { getItem: name => saved.get(name), setItem: (name, value) => saved.set(name, value) },
    };
    vm.createContext(context);
    const source = fs.readFileSync(path.join(__dirname, '../src/voice/voiceManager.ts'), 'utf8')
        .replace(/^import .*\n/gm, '')
        .replace('export class VoiceManager', 'class VoiceManager')
        .replace('export const VOICE = new VoiceManager();', 'globalThis.VoiceManager = VoiceManager;');
    vm.runInContext(ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText, context);
    const voice = new context.VoiceManager();
    voice.status = 'connected';
    voice.microphoneEnabled = true;
    voice.session = { sendGroups: ['lobby'], forcedSilenced: false };
    function dispatch(type, code, modifiers = {}) {
        const event = { ...key(code, modifiers), target: new Element(), repeat: false, isComposing: false, defaultPrevented: false, preventDefault() {}, ...modifiers };
        listeners.get(type)(event);
    }
    return { voice, dispatch, listeners, context };
}

test('captures and labels combinations with multiple modifiers', () => {
    const event = key('KeyM', { shiftKey: true, ctrlKey: true });
    assert.equal(keybind.keybindFromEvent(event), 'Control+Shift+KeyM');
    assert.equal(keybind.formatKeybind('Control+Shift+KeyM'), 'Ctrl+Shift+M');
    assert.equal(keybind.formatKeybind('KeyV'), 'V');
    assert.equal(keybind.formatKeybind('Digit1'), '1');
    assert.equal(keybind.keybindFromEvent(key('ShiftLeft', { shiftKey: true })), 'ShiftLeft');
});

test('matching requires the primary key and exactly the bound modifiers', () => {
    assert.equal(keybind.matchesKeybind('Shift+KeyM', key('KeyM', { shiftKey: true })), true);
    assert.equal(keybind.matchesKeybind('Shift+KeyM', key('KeyM')), false);
    assert.equal(keybind.matchesKeybind('Shift+KeyM', key('KeyM', { shiftKey: true, ctrlKey: true })), false);
    assert.equal(keybind.matchesKeybind('KeyV', key('KeyV')), true);
    assert.equal(keybind.matchesKeybind('ShiftLeft', key('ShiftLeft', { shiftKey: true })), true);
});

test('Shift+M toggles mute once and ignores repeats and unmodified M', () => {
    const { voice, dispatch } = setup('toggleMute', 'Shift+KeyM');
    dispatch('keydown', 'KeyM');
    assert.equal(voice.selfMuted, true);
    dispatch('keydown', 'KeyM', { shiftKey: true });
    assert.equal(voice.selfMuted, false);
    dispatch('keydown', 'KeyM', { shiftKey: true, repeat: true });
    assert.equal(voice.selfMuted, false);
    dispatch('keydown', 'KeyM', { shiftKey: true });
    assert.equal(voice.selfMuted, true);
});

for (const modifier of ['ctrlKey', 'altKey', 'metaKey']) {
    test(`supports ${modifier} in voice shortcuts`, () => {
        const binding = keybind.keybindFromEvent(key('KeyM', { [modifier]: true }));
        const { voice, dispatch } = setup('toggleMute', binding);
        dispatch('keydown', 'KeyM', { [modifier]: true });
        assert.equal(voice.selfMuted, false);
    });
}

for (const released of ['KeyM', 'ShiftLeft', 'ShiftRight']) {
    test(`push-to-talk stops when ${released} is released`, () => {
        const { voice, dispatch } = setup('pushToTalk', 'Shift+KeyM');
        voice.selfMuted = false;
        dispatch('keydown', 'KeyM', { shiftKey: true });
        assert.equal(voice.talking, true);
        dispatch('keyup', released, { shiftKey: released === 'KeyM' });
        assert.equal(voice.talking, false);
    });
}

test('unrelated key release keeps push-to-talk held, extra modifiers stop it', () => {
    const { voice, dispatch } = setup('pushToTalk', 'Shift+KeyM');
    voice.selfMuted = false;
    dispatch('keydown', 'KeyM', { shiftKey: true });
    dispatch('keyup', 'KeyX', { shiftKey: true });
    assert.equal(voice.talking, true);
    dispatch('keydown', 'ControlLeft', { shiftKey: true, ctrlKey: true });
    assert.equal(voice.talking, false);
});

test('window blur releases push-to-talk', () => {
    const { voice, dispatch, listeners } = setup('pushToTalk', 'Shift+KeyM');
    voice.selfMuted = false;
    dispatch('keydown', 'KeyM', { shiftKey: true });
    listeners.get('blur')();
    assert.equal(voice.talking, false);
});

test('typing, composition, and already handled shortcuts do not trigger voice', () => {
    const { voice, dispatch, context } = setup('toggleMute', 'Shift+KeyM');
    for (const flags of [{ defaultPrevented: true }, { isComposing: true }]) {
        dispatch('keydown', 'KeyM', { shiftKey: true, ...flags });
        assert.equal(voice.selfMuted, true);
    }
    vm.runInContext('globalThis.input = new Element(); input.editable = true;', context);
    dispatch('keydown', 'KeyM', { shiftKey: true, target: context.input });
    assert.equal(voice.selfMuted, true);
});

test('existing single-key bindings and stored combinations keep working', () => {
    const { voice, dispatch, context } = setup('toggleMute', 'KeyV');
    dispatch('keydown', 'KeyV');
    assert.equal(voice.selfMuted, false);
    voice.setSettings({ key: 'Control+Shift+KeyM' });
    assert.equal(new context.VoiceManager().settings.key, 'Control+Shift+KeyM');
});
