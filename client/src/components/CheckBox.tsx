import React from "react"
import Icon from "./Icon"
import "./checkBox.css"

export default function CheckBox(props: {
    checked: boolean,
    onChange: (checked: boolean) => void
}) {
    // TODO: Change to button
    return <label
        className="checkbox"
        onClick={()=>{
            props.onChange(!props.checked)
        }}
    >
        <Icon>{props.checked ? "check" : "close"}</Icon>
    </label>
}