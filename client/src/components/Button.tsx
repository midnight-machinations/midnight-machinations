import React, { useEffect, useMemo, useRef, ReactElement, useState, forwardRef, useContext } from "react";
import "./button.css";
import ReactDOM from "react-dom/client";
import { THEME_CSS_ATTRIBUTES } from "..";
import { CtrlPressedContext } from "../menu/Anchor";
import Popover from "./Popover";
import { dropdownPlacementFunction } from "./Select";
import { WikiArticleLink } from "./WikiArticleLink";
import WikiArticle from "./WikiArticle";
import WikiArticleTooltip, { getArticleTooltip } from "./WikiArticleTooltip";

export type ButtonProps<R> = Omit<JSX.IntrinsicElements['button'], 'onClick' | 'ref'> & {
    onClick?: (event: React.MouseEvent<HTMLButtonElement, MouseEvent>) => (R | void | Promise<R | void>)
    highlighted?: boolean,
    pressedChildren?: (result: R) => React.ReactNode
    pressedText?: (result: R) => React.ReactNode
    tooltip?: JSX.Element | WikiArticleLink
};

function reconcileProps<R>(props: ButtonProps<R>): JSX.IntrinsicElements['button'] {
    const newProps = {...props};
    delete newProps.onClick;
    delete newProps.highlighted;
    delete newProps.pressedChildren;
    delete newProps.pressedText;
    delete newProps.tooltip;

    return newProps;
}

const POPUP_TIMEOUT_MS = 1000;

export function Button<R>(props: ButtonProps<R>): ReactElement {
    return <RawButton {...props} />
}

const RawButton = forwardRef<HTMLButtonElement, ButtonProps<any>>(function RawButton<R>(props: ButtonProps<R>, passedRef: React.ForwardedRef<HTMLButtonElement>): ReactElement {
    const [success, setSuccess] = useState<R | "unclicked">("unclicked");
    const ref = useRef<HTMLButtonElement>(null);

    useEffect(() => {
        if (typeof passedRef === "function") {
            passedRef(ref.current);
        } else if (passedRef) {
            passedRef.current = ref.current
        }
    }, [props, ref, passedRef]);

    const popupContainer = useRef<HTMLDivElement>(document.createElement('div'));

    let lastTimeout: NodeJS.Timeout | null = null;

    const showPopup = (content: React.ReactNode) => {
        if (ref.current === null) return;
        const root = ReactDOM.createRoot(popupContainer.current);
        root.render(<ButtonPopup button={ref.current}>{content}</ButtonPopup>);
        document.body.appendChild(popupContainer.current);
    }

    const hidePopup = () => {
        if (document.body.contains(popupContainer.current)) {
            document.body.removeChild(popupContainer.current)
        }
    }
    
    const children = useMemo(() => {
        if (success === "unclicked" || props.pressedChildren === undefined) return props.children;
        return props.pressedChildren(success) || props.children;
    }, [props, success]);
    
    const isCtrlPressed = useContext(CtrlPressedContext) ?? false;
    
    const articleTooltip = React.useMemo(() => {
        if (props.tooltip === undefined) return null;
        if (typeof props.tooltip !== "string") return props.tooltip;

        if (isCtrlPressed === true) {
            return <WikiArticle noLinks={true} article={props.tooltip} className="wiki-article-tooltip" />;
        } else {
            const tooltip = getArticleTooltip(props.tooltip);
            if (tooltip === null) {
                return null;
            }
            return <WikiArticleTooltip tooltip={tooltip} />;
        }
    }, [isCtrlPressed, props.tooltip]);

    const [hovering, setHovering] = React.useState<boolean>(false);

    const handleFocus = (event: any) => {
        setHovering(true);
    };

    const handleUnfocus = (event: any) => {
        setHovering(false);
    };

    return <>
        <button {...reconcileProps(props)} ref={ref}
            className={
                "button " + (props.className ?? "") + (props.highlighted ? " highlighted" : "")
            }
            onClick={async e => {
                if (props.onClick) {
                    const result = await props.onClick(e);
                    if (result === undefined) return;

                    setSuccess(result);
                    if (props.pressedText !== undefined) showPopup(props.pressedText(result))

                    if (lastTimeout) clearTimeout(lastTimeout);
                    lastTimeout = setTimeout(() => {
                        setSuccess("unclicked")
                        hidePopup();
                    }, POPUP_TIMEOUT_MS);
                }
            }}
            onMouseEnter={handleFocus}
            onMouseLeave={handleUnfocus}
            onFocus={handleFocus}
            onBlur={handleUnfocus}
        >{children}</button>
        {articleTooltip !== null && <Popover
            open={hovering && articleTooltip !== null}
            setOpenOrClosed={setHovering}
            anchorForPositionRef={ref}
            onRender={(popover, anchor) => dropdownPlacementFunction(popover, anchor, null)}
            className="wiki-article-tooltip-popover"
        >
            {articleTooltip}
        </Popover>}
    </>
})

export { RawButton };

function ButtonPopup(props: { children: React.ReactNode, button: HTMLButtonElement }): ReactElement {
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if ((ref.current) === null) return;

        const buttonBounds = props.button.getBoundingClientRect();
        ref.current.style.top = `${buttonBounds.bottom}px`;
        ref.current.style.left = `${(buttonBounds.left + buttonBounds.width / 2)}px`;
        THEME_CSS_ATTRIBUTES.forEach(prop => {
            if ((ref.current) === null) return;
            ref.current.style.setProperty(`--${prop}`, getComputedStyle(props.button).getPropertyValue(`--${prop}`))
        })
    });
    
    return <div className="button-popup" ref={ref}>{props.children}</div>
}