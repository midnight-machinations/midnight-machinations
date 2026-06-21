import { ReactElement } from "react";
import "./icon.css"

export default function Icon(props: JSX.IntrinsicElements['span'] & { size?: "normal" | "small" | "tiny" }): ReactElement {
    let sizeClassName;
    switch (props.size) {
        case undefined:
        case "normal":
            sizeClassName = "icon"
        break
        default:
            sizeClassName = `icon-${props.size}`
        break
    }
    return <span {...props} className={"material-symbols-rounded " + (props.className ?? "") + (" " + sizeClassName)}/>
}