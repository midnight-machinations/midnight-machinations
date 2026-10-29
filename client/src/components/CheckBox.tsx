import React from "react"
import Icon from "./Icon"
import "./checkBox.css"

export default function CheckBox(props: {
    checked: boolean,
    onChange: (checked: boolean) => void,
    disabled?: boolean,
    ariaLabel?: string,
}) {
    // TODO: Change to button
    return <label
        className="checkbox"
        role="checkbox"
        aria-checked={props.checked}
        aria-disabled={props.disabled || undefined}
        aria-label={props.ariaLabel}
        tabIndex={props.disabled ? -1 : 0}
        onClick={()=>{
            if (!props.disabled) props.onChange(!props.checked)
        }}
        onKeyDown={event => {
            if (event.key !== " " && event.key !== "Enter") return;
            event.preventDefault();
            if (!props.disabled && !event.repeat) props.onChange(!props.checked);
        }}
    >
        <Icon>{props.checked ? "check" : "close"}</Icon>
    </label>
}
