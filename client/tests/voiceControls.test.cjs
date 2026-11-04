const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');

const voice = {
    status: 'disconnected', roomEnabled: true, error: '', settings: { mode: 'openMic' }, microphoneEnabled: false,
    session: { selfId: 1, sendGroups: ['all'], receiveGroups: ['all'], forcedSilenced: false,
        peers: [{ id: 1, name: 'Me' }, { id: 2, name: 'Alice' }, { id: 3, name: '<Bob>' }] },
    group: () => ({ deafened: false }), person: () => ({ volume: 1 }),
};
const context = {
    React: { createElement: (type, props, ...children) => ({ type, props, children }) },
    VOICE: voice, useSyncExternalStore() {},
    translate: key => key === 'voice.groups' ? 'Deafen by group' : key,
    encodeString: value => value.replaceAll('<', '&lt;').replaceAll('>', '&gt;'),
    StyledText: function StyledText() {}, SpeakingIndicator: function SpeakingIndicator() {}, Icon: function Icon() {},
};
const source = fs.readFileSync(path.join(__dirname, '../src/voice/VoiceControls.tsx'), 'utf8')
    .replace(/^import .*\n/gm, '')
    .replace(/export default /g, '').replace(/export /g, '');
vm.createContext(context);
vm.runInContext(ts.transpileModule(source, { compilerOptions: { jsx: ts.JsxEmit.React } }).outputText, context);
function find(node, predicate) {
    if (!node) return [];
    if (Array.isArray(node)) return node.flatMap(child => find(child, predicate));
    return [...(predicate(node) ? [node] : []), ...find(node.children, predicate)];
}

for (const status of ['disconnected', 'connecting', 'error']) {
    test(`individual volumes are absent while ${status}`, () => {
        voice.status = status;
        const tree = context.VoiceControls();
        assert.equal(find(tree, node => node.type === 'details').length, 0);
        assert.equal(find(tree, node => node.type === 'input' && node.props?.type === 'range').length, 0);
    });
}

test('connected voice shows volume controls for other players only', () => {
    voice.status = 'connected';
    const tree = context.VoiceControls();
    assert.equal(find(tree, node => node.type === 'details').length, 1);
    assert.equal(find(tree, node => node.type === 'input' && node.props?.type === 'range').length, 2);
    assert.equal(find(tree, node => node.type === context.StyledText && node.children[0] === '&lt;Bob&gt;').length, 1);
});

test('group deafen label uses StyledText', () => {
    assert.equal(find(context.VoiceControls(), node => node.type === context.StyledText && node.children[0] === 'Deafen by group').length, 1);
});
