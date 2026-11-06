import type { ReactElement, ReactNode } from "react";
import "./counter.css";

export default function Counter(
    props: Readonly<{
        max: number;
        current: number;
        children?: ReactNode;
    }>,
): ReactElement {
    const circles = [];

    for (let i = 0; i < Math.max(props.max, props.current); i++) {
        const filled = i < props.current ? "filled" : "empty";

        circles.push(<div key={filled + i} className={`counter-circle counter-circle-${filled}`} />);
    }
    return (
        <div className="counter">
            <div>{props.children}</div>
            <div className="counter-count">{circles}</div>
        </div>
    );
}
