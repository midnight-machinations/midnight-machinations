import type { KeyboardEvent } from "react";

/** Props that let a non-button element act like a button: focusable, and activated by Enter or Space. */
export function buttonProps(onActivate: () => void) {
    return {
        role: "button",
        tabIndex: 0,
        onClick: onActivate,
        onKeyDown: (event: KeyboardEvent) => {
            // Ignore keys pressed in a child control, such as typing a space in an input
            if (event.target !== event.currentTarget) return;
            if (event.key !== "Enter" && event.key !== " ") return;
            event.preventDefault();
            onActivate();
        },
    } as const;
}
