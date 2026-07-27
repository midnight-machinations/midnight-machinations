import React, { ReactElement, ReactNode, useContext, useEffect, useMemo, useRef } from "react";
import ReactDOM from "react-dom/client";
import { THEME_CSS_ATTRIBUTES } from "..";
import { AnchorControllerContext, MobileContext, TooltipContext } from "../menu/Anchor";
import { MenuControllerContext } from "../menu/game/GameScreen";
import { GameModeContext } from "./gameModeSettings/GameModesEditor";

export default function Popover<T extends HTMLElement = HTMLElement>(props: Readonly<{
    open: boolean,
    children: ReactNode,
    setOpenOrClosed: (open: boolean) => void,
    onRender?: (popoverElement: HTMLDivElement, anchorElement?: T | undefined) => void
    anchorForPositionRef?: React.RefObject<T | null>,
    className?: string,
    doNotCloseOnOutsideClick?: boolean
}>): ReactElement {
    const { onRender, anchorForPositionRef, children, open } = props;

    const thisRef = useRef<HTMLDivElement>(null);
    const popoverRef = useRef<HTMLDivElement>(document.createElement('div'));

    const popoverRoot = useMemo(() => {
        const popoverElement = popoverRef.current;
        // eslint-disable-next-line react-hooks/refs
        popoverElement.style.position = "absolute";

        // eslint-disable-next-line react-hooks/refs
        document.body.appendChild(popoverElement);
        // eslint-disable-next-line react-hooks/refs
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
    const tooltipContext = useContext(TooltipContext);

    //open and set position
    useEffect(() => {
        const popoverElement = popoverRef.current;
        const anchorElement = anchorForPositionRef?.current;

        if (open) {
            popoverRoot.render(
                <AnchorControllerContext.Provider value={anchorControllerContext}>
                    <MenuControllerContext.Provider value={menuControllerContext}>
                        <GameModeContext.Provider value={gameModeContext}>
                            <MobileContext.Provider value={mobileContext}>
                                <TooltipContext.Provider value={tooltipContext}>
                                    {children}
                                </TooltipContext.Provider>
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
                
                if (onRender) {
                    onRender(popoverElement, anchorElement ?? undefined)
                }
            })
        } else {
            popoverElement.hidden = true;
        }
    }, [children, onRender, anchorForPositionRef, open, popoverRoot, anchorControllerContext, menuControllerContext, gameModeContext, mobileContext, tooltipContext]);

    // Resize when children change
    useEffect(() => {
        setTimeout(() => {
            if (onRender) {
                onRender(popoverRef.current, anchorForPositionRef?.current ?? undefined)
            }
        })
    }, [anchorForPositionRef, children, onRender, popoverRef])

    //close on click outside
    useEffect(() => {
        if (props.doNotCloseOnOutsideClick) {
            return;
        }

        const handleClickOutside = (event: MouseEvent) => {
            if (
                !popoverRef.current?.contains(event.target as Node) &&
                !props.anchorForPositionRef?.current?.contains(event.target as Node)
                && props.open
            ) {
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

    return <div ref={thisRef} hidden={true}/>
}