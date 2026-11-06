const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');

const react = { useRef: value => ({ current: value }), useMemo: fn => fn() };
const library = { exports: {} };
vm.runInNewContext(fs.readFileSync(require.resolve('react-swipeable'), 'utf8'), {
    exports: library.exports, require: name => {
        assert.equal(name, 'react');
        return react;
    },
});

class Element {
    parentElement = null;
    scrollWidth = 100;
    clientWidth = 100;
    overflowX = 'hidden';
    control = false;
    listeners = new Map();
    closest() { return this.control ? this : this.parentElement?.closest() ?? null; }
    addEventListener(name, listener) { this.listeners.set(name, listener); }
    removeEventListener(name) { this.listeners.delete(name); }
}

const source = fs.readFileSync(path.join(__dirname, '../src/menu/game/useMenuSwipe.ts'), 'utf8');
const compiled = ts.transpileModule(source, { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;

function setup(enabled = true) {
    const exports = {};
    let selected = false;
    vm.runInNewContext(compiled, {
        exports,
        require: name => name === 'react' ? react : library.exports,
        window: { getSelection: () => ({ isCollapsed: !selected }) },
        Element,
        getComputedStyle: element => ({ overflowX: element.overflowX }),
    });
    const calls = [];
    const handlers = exports.useMenuSwipe(enabled, right => calls.push(right));
    const root = new Element();
    const target = new Element();
    target.parentElement = root;
    handlers.ref(root);
    function touch(name, x, y, time = 100, count = 1) {
        const event = {
            touches: name === 'touchend' ? [] : Array.from({ length: count }, () => ({ clientX: x, clientY: y })),
            target, currentTarget: root, timeStamp: time,
        };
        if (name === 'touchstart') handlers.onTouchStartCapture(event);
        if (name === 'touchcancel') handlers.onTouchCancelCapture(event);
        root.listeners.get(name)?.(event);
    }
    return { calls, touch, target, setSelection: value => { selected = value; }, next: exports.getNextSwipeMenu };
}

test('horizontal swipes switch once in either direction', () => {
    const { touch, calls } = setup();
    touch('touchstart', 150, 100, 0);
    touch('touchmove', 70, 105, 100);
    touch('touchend', 70, 105, 120);
    touch('touchstart', 70, 100, 200);
    touch('touchmove', 150, 105, 300);
    touch('touchend', 150, 105, 320);
    assert.deepEqual(calls, [false, true]);
});

for (const [name, move] of [
    ['tap', [150, 100]], ['short swipe', [125, 100]],
    ['vertical scroll down', [145, 200]], ['vertical scroll up', [145, 0]],
    ['diagonal scroll', [70, 165]],
]) {
    test(`${name} does not switch tabs`, () => {
        const { touch, calls } = setup();
        touch('touchstart', 150, 100, 0);
        touch('touchmove', ...move, 100);
        touch('touchend', ...move, 120);
        assert.deepEqual(calls, []);
    });
}

for (const kind of ['desktop', 'control', 'horizontal scroller', 'selection', 'multitouch', 'cancel', 'vertical first', 'too slow']) {
    test(`${kind} does not switch tabs`, () => {
        const s = setup(kind !== 'desktop');
        if (kind === 'control') s.target.control = true;
        if (kind === 'horizontal scroller') { s.target.scrollWidth = 300; s.target.overflowX = 'auto'; }
        if (kind === 'selection') s.setSelection(true);
        s.touch('touchstart', 150, 100, 0);
        if (kind === 'multitouch') s.touch('touchstart', 150, 100, 20, 2);
        if (kind === 'vertical first') s.touch('touchmove', 148, 120, 20);
        s.touch('touchmove', 70, 105, 100);
        if (kind === 'cancel') s.touch('touchcancel', 70, 105, 110);
        s.touch('touchend', 70, 105, kind === 'too slow' ? 900 : 120);
        assert.deepEqual(s.calls, []);
    });
}

test('navigation follows tab order, wraps, and stops safely when no alternative exists', () => {
    const { next } = setup();
    const menus = ['chat', 'wiki', 'players'];
    assert.equal(next(menus, ['chat'], false), 'wiki');
    assert.equal(next(menus, ['chat'], true), 'players');
    assert.equal(next(menus, ['players'], false), 'chat');
    assert.equal(next(menus, ['chat', 'wiki'], false), 'players');
    assert.equal(next(menus, menus, false), undefined);
    assert.equal(next(['chat'], ['chat'], false), undefined);
    assert.equal(next(menus, [], false), undefined);
});

test('dragging back to the starting point cancels a swipe', () => {
    const { touch, calls } = setup();
    touch('touchstart', 150, 100, 0);
    touch('touchmove', 70, 100, 100);
    touch('touchmove', 140, 100, 150);
    touch('touchend', 140, 100, 170);
    assert.deepEqual(calls, []);
});

test('closing and opening tabs in the same batch preserves both changes', () => {
    let source = fs.readFileSync(path.join(__dirname, '../src/menu/game/GameScreen.tsx'), 'utf8')
        .replace(/^import .*\r?\n/gm, '')
        .replace(/^export \{.*\}\r?\n/gm, '')
        .replace(/export default /g, '').replace(/export /g, '');
    let controller;
    let stateIndex = 0;
    const states = [];
    const updates = [];
    const context = {
        createContext: () => ({}), useCallback: fn => fn, useEffect() {},
        useState: initial => {
            const index = stateIndex++;
            states[index] = initial;
            return [initial, update => updates.push(() => {
                states[index] = typeof update === 'function' ? update(states[index]) : update;
            })];
        },
        GAME_MANAGER: { state: { stateType: 'game', clientState: { type: 'player' } }, invokeStateListeners() {} },
    };
    for (const name of ['ChatMenu', 'AbilityMenu', 'WillMenu', 'PlayerListMenu', 'GraveyardMenu', 'WikiMenu']) context[name] = () => {};
    vm.createContext(context);
    vm.runInContext(ts.transpileModule(source, { compilerOptions: { jsx: ts.JsxEmit.React } }).outputText, context);
    context.useMenuController(3, { ChatMenu: true, WikiMenu: false, PlayerListMenu: true }, () => controller, value => { controller = value; });
    controller.closeMenu('PlayerListMenu');
    controller.openMenu('WikiMenu');
    updates.forEach(update => update());
    assert.equal(states[0].ChatMenu, true);
    assert.equal(states[0].PlayerListMenu, false);
    assert.equal(states[0].WikiMenu, true);
});
