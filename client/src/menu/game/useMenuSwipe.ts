import { type TouchEvent, useRef } from "react";
import { type SwipeEventData, useSwipeable } from "react-swipeable";

export function getNextSwipeMenu<T>(menus: readonly T[], open: readonly T[], right: boolean): T | undefined {
    if (open.length === 0) return;
    const current = menus.indexOf(open[open.length - 1]);
    if (current < 0) return;
    for (let step = 1; step < menus.length; step++) {
        const index = (current + (right ? -step : step) + menus.length) % menus.length;
        if (!open.includes(menus[index])) return menus[index];
    }
}

function hasSelection() {
    return window.getSelection()?.isCollapsed === false;
}

function isTouchControl(target: EventTarget | null, root: EventTarget | null) {
    if (!(target instanceof Element)) return true;
    if (
        target.closest(
            "input, textarea, select, button, a, [contenteditable]:not([contenteditable=false]), [role=slider], [role=combobox], .custom-select",
        )
    )
        return true;
    // Leave horizontally scrollable tables and other nested scrollers alone.
    for (let element: Element | null = target; element && element !== root; element = element.parentElement) {
        if (element.scrollWidth > element.clientWidth && /^(auto|scroll)$/.test(getComputedStyle(element).overflowX))
            return true;
    }
    return false;
}

export function useMenuSwipe(enabled: boolean, onSwipe: (right: boolean) => void) {
    const blocked = useRef(false);
    const finish = (data: SwipeEventData, right: boolean) => {
        if (enabled && !blocked.current && !hasSelection() && data.absX >= 48 && data.absX >= data.absY * 1.5)
            onSwipe(right);
    };
    const handlers = useSwipeable({
        trackTouch: enabled,
        trackMouse: false,
        delta: { left: 48, right: 48, up: 10, down: 10 },
        swipeDuration: 700,
        preventScrollOnSwipe: false,
        onTouchStartOrOnMouseDown: ({ event }) => {
            blocked.current = hasSelection() || isTouchControl(event.target, event.currentTarget);
        },
        onSwiping: ({ dir }) => {
            if (dir === "Up" || dir === "Down") blocked.current = true;
        },
        onSwipedLeft: (data) => finish(data, false),
        onSwipedRight: (data) => finish(data, true),
    });
    return {
        ...handlers,
        onTouchStartCapture(event: TouchEvent<HTMLDivElement>) {
            if (event.touches.length !== 1) blocked.current = true;
        },
        onTouchCancelCapture() {
            blocked.current = true;
        },
    };
}
