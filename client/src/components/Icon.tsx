import type { ReactElement } from "react";
import "./icon.css";

export default function Icon(
    props: JSX.IntrinsicElements["span"] & { size?: "normal" | "small" | "tiny" },
): ReactElement {
    const sizeClassName = props.size === undefined || props.size === "normal" ? "icon" : `icon-${props.size}`;
    return <span {...props} className={`material-symbols-rounded ${props.className ?? ""} ${sizeClassName}`} />;
}
