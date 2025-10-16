import React, { useMemo, useState } from "react";
import "./dragAndDrop.css"

export type DragHandleProps = {
    className: string;
    draggable: boolean;
    onDragStart: (e: React.DragEvent) => void;
    onDragEnd: (e: React.DragEvent) => void;
    onMouseUp?: (e: React.MouseEvent) => void;
}

// Adapted from: https://github.com/atlassian/react-beautiful-dnd/issues/316#issuecomment-1860490084
export function DragAndDrop<T>(props: Readonly<{ 
    items: T[], 
    onDragEnd: (newItems: T[]) => void,
    disabled?: boolean,
} & (
    {
        dragHandle?: false,
        render: (item: T, index: number) => React.ReactNode,
    } | {
        dragHandle: true,
        render: (item: T, index: number, dragHandleProps: DragHandleProps) => React.ReactNode,
    }
)>): React.ReactElement {
    const [temporaryItems, setTemporaryItems] = useState<T[] | null>(null);
    const [draggedItem, setDraggedItem] = useState<T | null>(null);

    const renderedItems = temporaryItems ?? props.items;

    return <>
        {renderedItems.map((item, index) => <DragAndDropItem<T>
            key={index} 
            item={item} 
            index={index} 
            draggedItem={draggedItem} 
            setDraggedItem={setDraggedItem} 
            renderedItems={renderedItems} 
            dragHandle={props.dragHandle}
            render={props.render}
            setTemporaryItems={setTemporaryItems} 
            onDragEnd={props.onDragEnd}
        />)}
    </>
}

function DragAndDropItem<T>(props: Readonly<{
    item: T,
    index: number,
    draggedItem: T | null,
    setDraggedItem: (item: T | null) => void,
    renderedItems: T[],
    disabled?: boolean,
    dragHandle?: boolean,
    render: (item: T, index: number, dragHandleProps: DragHandleProps) => React.ReactNode,
    setTemporaryItems: (items: T[] | null) => void,
    onDragEnd: (newItems: T[]) => void,
}>) {
    const { item, index, draggedItem, setDraggedItem, renderedItems, setTemporaryItems, render, disabled, onDragEnd } = props;

    const dragHandleProps: DragHandleProps = useMemo(() => ({
        className: (disabled ? "" : "draggable"),
        draggable: !disabled,
        onDragStart: () => setDraggedItem(item),
        onDragEnd: () => {
            onDragEnd(renderedItems);
            setDraggedItem(null);
            setTemporaryItems(null);
        },
    }), [disabled, item, setDraggedItem, renderedItems, setTemporaryItems, onDragEnd]);

    const children = useMemo(() => {
        return render(item, index, dragHandleProps);
    }, [render, item, index, dragHandleProps]);

    return <div
        {...(props.dragHandle ? {} : dragHandleProps)}
        className={((disabled || props.dragHandle) ? "" : "draggable") + (item === draggedItem ? " dragged" : "")}
        onPointerMove={props.dragHandle ? (
            (e) => {
                if (e.buttons !== 1) { // If mouse button is not held down
                    onDragEnd(renderedItems);
                    setDraggedItem(null);
                    setTemporaryItems(null);
                }
            }
        ) : undefined}
        onDragOver={(e) => {
            e.preventDefault();
            if (draggedItem === null || draggedItem === item) {
                return;
            }
            const currentIndex = renderedItems.indexOf(draggedItem);
            const targetIndex = renderedItems.indexOf(item);
            
            if (currentIndex !== -1 && targetIndex !== -1) {
                const newItems = [...renderedItems];
                newItems.splice(currentIndex, 1);
                newItems.splice(targetIndex, 0, draggedItem);
                setTemporaryItems(newItems);
            }
        }}
    >
        {children}
    </div>
}