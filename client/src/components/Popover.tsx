import React, { ReactElement, ReactNode, useContext, useEffect, useMemo, useRef } from "react";
import ReactDOM from "react-dom/client";
import { THEME_CSS_ATTRIBUTES } from "..";
import { AnchorControllerContext, MobileContext } from "../menu/Anchor";
import { MenuControllerContext } from "../menu/game/GameScreen";
import { GameModeContext } from "./gameModeSettings/GameModesEditor";

export default function Popover<T extends HTMLElement = HTMLElement>(props: Readonly<{
    open: boolean,
    children: ReactNode,
    setOpenOrClosed: (open: boolean) => void,
    onRender?: (popoverElement: HTMLDivElement, anchorElement?: T | undefined) => void
    anchorForPositionRef?: React.RefObject<T>,
    className?: string,
    doNotCloseOnOutsideClick?: boolean
}>): ReactElement {
    const thisRef = useRef<HTMLDivElement>(null);
    const popoverRef = useRef<HTMLDivElement>(document.createElement('div'));

    const popoverRoot = useMemo(() => {
        const popoverElement = popoverRef.current;
        popoverElement.style.position = "absolute";

        document.body.appendChild(popoverElement);
        return ReactDOM.createRoot(popoverElement);
    }, [])

    //set ref
    useEffect(() => {
        const initialPopover = popoverRef.current;
        return () => {
            setTimeout(() => {
                popoverRoot.unmount();
            })
            initialPopover.remove();
            
            popoverRef.current = document.createElement('div');
        }
    }, [popoverRoot])

    //match css styles
    useEffect(() => {
        const styleCopyFrom = props.anchorForPositionRef?.current ?? thisRef.current;
        const popoverElement = popoverRef.current;
        
        if (styleCopyFrom) {
            // Match styles
            THEME_CSS_ATTRIBUTES.forEach(prop => {
                popoverElement.style.setProperty(`--${prop}`, getComputedStyle(styleCopyFrom).getPropertyValue(`--${prop}`))
            })

            popoverElement.className = 'popover ' + (props.className ?? '')
        }
    }, [props.anchorForPositionRef, props.className])

    // This is for the popover's anchor, not the element named Anchor
    const [anchorLocation, setAnchorLocation] = React.useState(() => {
        const bounds = props.anchorForPositionRef?.current?.getBoundingClientRect();

        if (bounds) {
            return { top: bounds.top, left: bounds.left }
        } else {
            return {top: 0, left: 0}
        }
    });

    //close on scroll
    useEffect(() => {
        const listener = () => {
            const bounds = props.anchorForPositionRef?.current?.getBoundingClientRect();
            if (
                bounds &&
                props.open &&
                (
                    anchorLocation.top !== bounds?.top || 
                    anchorLocation.left !== bounds?.left
                )
            )
            props.setOpenOrClosed(false);
        };
        
        window.addEventListener("scroll", listener, true);
        window.addEventListener("resize", listener);
        return () => {
            window.removeEventListener("scroll", listener, true);
            window.removeEventListener("resize", listener);
        }
    })

    const anchorControllerContext = useContext(AnchorControllerContext);
    const menuControllerContext = useContext(MenuControllerContext);
    const gameModeContext = useContext(GameModeContext);
    const mobileContext = useContext(MobileContext);

    //open and set position
    useEffect(() => {
        const popoverElement = popoverRef.current;
        const anchorElement = props.anchorForPositionRef?.current;

        if (props.open) {
            popoverRoot.render(
                <AnchorControllerContext.Provider value={anchorControllerContext}>
                    <MenuControllerContext.Provider value={menuControllerContext}>
                        <GameModeContext.Provider value={gameModeContext}>
                            <MobileContext.Provider value={mobileContext}>
                                {props.children}
                            </MobileContext.Provider>
                        </GameModeContext.Provider>
                    </MenuControllerContext.Provider>
                </AnchorControllerContext.Provider>
            );

            if (anchorElement) {
                const anchorBounds = anchorElement.getBoundingClientRect();

                setAnchorLocation({top: anchorBounds.top, left: anchorBounds.left});
            }

            setTimeout(() => {
                popoverElement.hidden = false;
                
                if (props.onRender) {
                    props.onRender(popoverElement, anchorElement ?? undefined)
                }
            })
        } else {
            popoverElement.hidden = true;
        }
    }, [props, popoverRoot, anchorControllerContext, menuControllerContext, gameModeContext, mobileContext]);

    // This is to make sure popovers which are "children" of closed popovers don't open
    useEffect(() => {
        const checkAnchorOpen = () => {
            const anchorElement = props.anchorForPositionRef?.current;

            const anchorRootIsOpen = (() => {
                let current: HTMLElement | null | undefined = anchorElement;
                while (current !== null && current !== undefined) {
                    if (current.hidden) return false;
                    current = current.parentElement;
                }
                return true;
            })()

            if (!anchorRootIsOpen) {
                props.setOpenOrClosed(false);
            }
        }

        // Hopefully all closings of popovers are caused by clicks.
        window.addEventListener("click", checkAnchorOpen)
        return () => window.removeEventListener("click", checkAnchorOpen);
    }, [props.anchorForPositionRef])

    //close on click outside
    useEffect(() => {
        if (props.doNotCloseOnOutsideClick) {
            return;
        }

        const handleClickOutside = (event: MouseEvent) => {
            if (!popoverRef.current?.contains(event.target as Node) && props.open) {
                props.setOpenOrClosed(false);
            }
        };

        setTimeout(() => {
            document.addEventListener("click", handleClickOutside);
        })
        return () => {
            setTimeout(() => {
                document.removeEventListener("click", handleClickOutside);
            })
        }
    }, [props]);

    return <div ref={thisRef} />
}