import React from 'react';
import { createRoot } from 'react-dom/client';
import './index.css';
import Anchor from './menu/Anchor';
import { GameManager, createGameManager } from './game/gameManager';
import LoadingScreen from './menu/LoadingScreen';
import route from './routing';

export const DEV_ENV = process.env.NODE_ENV !== 'production';

export type Theme = "chat-menu-colors" | "player-list-menu-colors" | "will-menu-colors" | "role-specific-colors" | "graveyard-menu-colors" | "wiki-menu-colors"

const THEME_CSS_ATTRIBUTES = [
    'background-color', 'fade-color', 'primary-color', 'secondary-color', 
    'text-color', 'primary-border-color', 'primary-border-shadow-color', 
    'background-border-color', 'background-border-shadow-color', 
    'hover-color', 'focus-outline-color'
];

export { THEME_CSS_ATTRIBUTES }

const ROOT = createRoot(document.querySelector("#root")!);
const GAME_MANAGER: GameManager = createGameManager();
const TIME_PERIOD = 1000;
export default GAME_MANAGER;

setInterval(() => {
    GAME_MANAGER.tick(TIME_PERIOD);
}, TIME_PERIOD);

new MutationObserver(mutations => {
    for (const mutation of mutations) {
        if (mutation.type === "childList") {
            const elem = mutation.target as Element;

            for (const glitch of elem.querySelectorAll('.glitch')) {
                if (!glitch.hasAttribute('data-text')) {
                    glitch.setAttribute('data-text', glitch.textContent ?? "")
                }
            }
        }
    }
}).observe(document.body, { subtree: true, childList: true });

ROOT.render(
    <Anchor onMount={anchorController => route(anchorController, window.location)}>
        <LoadingScreen type="default"/>
    </Anchor>
);

export function find(text: string): RegExp {
    // Detect if iOS <= 16.3
    // https://bugs.webkit.org/show_bug.cgi?id=174931
    // https://stackoverflow.com/a/11129615
    if(
        /(iPhone|iPod|iPad)/i.test(navigator.userAgent) && 
        /OS ([2-9]_\d)|(1[0-5]_\d)|(16_[0-3])(_\d)? like Mac OS X/i.test(navigator.userAgent)
    ) { 
        // This won't work if a keyword starts with a symbol, or ends with a number.
        return RegExp(`\\b${regEscape(text)}(?!\\w)`, "gi");
    } else {
        // \p{L} matches any letter in any language. It's a unicode thing apparently.
        // Importantly, it doesn't match numbers, while \w does.
        return RegExp(`(?<!\\p{L})${regEscape(text)}(?!\\p{L})`, "gi");
    }
}

export function regEscape(text: string) {
    return text.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, '\\$&')
}

export function replaceMentions(rawText: string, playerNames: string[]) {
    let text = rawText;
    playerNames.forEach((player, i) => {
        text = text.replace(find(`@${i + 1}`), player);
    });
    playerNames.forEach((player, i) => {
        text = text.replace(find(`@${player}`), player);
    });
    return text;
}

export function modulus(n: number, m: number) {
    return ((n % m) + m) % m;
}
