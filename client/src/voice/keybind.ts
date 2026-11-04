type KeyEvent = Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">;

const MODIFIERS = [
    ["Control", "ctrlKey"],
    ["Alt", "altKey"],
    ["Shift", "shiftKey"],
    ["Meta", "metaKey"],
] as const;

function modifierForKey(code: string) {
    return MODIFIERS.find(([name]) => code === `${name}Left` || code === `${name}Right`)?.[0];
}

export function isModifierKey(code: string) {
    return modifierForKey(code) !== undefined;
}

export function keybindFromEvent(event: KeyEvent): string {
    const ownModifier = modifierForKey(event.code);
    return [...MODIFIERS.filter(([name, flag]) => event[flag] && name !== ownModifier).map(([name]) => name), event.code].join("+");
}

export function matchesKeybindModifiers(binding: string, event: KeyEvent): boolean {
    const parts = binding.split("+");
    const ownModifier = modifierForKey(parts[parts.length - 1]);
    return MODIFIERS.every(([name, flag]) => name === ownModifier || event[flag] === parts.includes(name));
}

export function matchesKeybind(binding: string, event: KeyEvent): boolean {
    return event.code === binding.split("+").at(-1) && matchesKeybindModifiers(binding, event);
}

export function keybindReleased(binding: string, event: KeyEvent): boolean {
    return event.code === binding.split("+").at(-1) || !matchesKeybindModifiers(binding, event);
}

export function formatKeybind(binding: string): string {
    return binding.split("+").map(code => {
        const modifier = modifierForKey(code) ?? code;
        if (modifier === "Control") return "Ctrl";
        if (isModifierKey(code)) return modifier;
        return code.replace(/^(Key|Digit)(?=\w$)/, "");
    }).join("+");
}
