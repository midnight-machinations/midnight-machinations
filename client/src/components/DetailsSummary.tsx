import { ReactElement, ReactNode, useMemo } from "react";
import React from "react";
import "./detailsSummary.css";
import Icon from "./Icon";

export default function DetailsSummary(props: Readonly<{
    summary?: ReactNode,
    dropdownArrow?: boolean,
    children?: ReactNode,
    defaultOpen?: boolean,
    open?: boolean,
    disabled?: boolean,
    onClick?: () => void
    className?: string
    summaryClassName?: string
}>): ReactElement {

    const [openState, setOpen] = React.useState(props.defaultOpen??false);

    const open = useMemo(() => {
        if(props.disabled) return false;
        if(props.open !== undefined) return props.open;
        return openState;
    }, [props.open, openState, props.disabled]);

    return <div className={"details-summary-container "+(props.className??"")}>
        <div className={"details-summary-summary-container"+(open ? " open" : "")}
            onClick={(e) => {
                if(props.disabled) return;
                setOpen(!open);
                if(props.onClick) props.onClick();
            }}
        >
            {(props.dropdownArrow === undefined || props.dropdownArrow === true) ? 
                ((props.disabled === undefined || props.disabled===false) ? 
                    <Icon>{open ? "keyboard_arrow_down" : "keyboard_arrow_right"}</Icon>
                 : 
                    <Icon>close</Icon>
                ) : 
                null
            }
            {props.summary}
        </div>
        {(props.children !== undefined && open) ? props.children : null}
    </div>
}