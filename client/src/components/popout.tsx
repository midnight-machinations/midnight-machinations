import React, { createContext, ReactNode, useEffect, useRef, useState } from "react";
import { ReactElement } from "react";
import { createPortal } from "react-dom";


export type PopoutContextType = {
    window: Window
}
export const PopoutContext = createContext<PopoutContextType | null>(null);

export default function Popout(props: {
    children: ReactNode,
    onClose?: ()=>void
}): ReactElement {
    const [parentDivInWindow, setContainer] = useState<HTMLDivElement | null>(null);
    const newWindow = useRef<Window | null>(null);

    useEffect(() => {
        // Create container element on client-side
        setContainer(document.createElement("div"));
    }, []);

    useEffect(() => {
        // When container is ready
        if (parentDivInWindow) {
            // Create window
            newWindow.current = window.open(
                "localhost:3000",
                "",
                "width=400,height=600,left=200,top=200"
            );

            // Append container
            if(newWindow.current){
                copyStyles(window.document, newWindow.current.document);
                newWindow.current.document.body.appendChild(parentDivInWindow);
                newWindow.current.onbeforeunload = function(){
                    if(props.onClose){
                        props.onClose()
                    }
                }
            }

            // Save reference to window for cleanup
            const curWindow = newWindow.current;

            // Return cleanup function
            return () => {if(curWindow)curWindow.close();}
        }
    }, [parentDivInWindow, props]);

    if(parentDivInWindow!==null){
        return createPortal(
            (<div style={{height:"100vh",width:"100vw"}}>
                <PopoutContext.Provider
                    value={{window: newWindow.current??window}}
                >
                    {props.children},
                </PopoutContext.Provider>
            </div>)
            ,
            parentDivInWindow
        );
    }
    return <></>;
};

function copyStyles(src: Document, dest: Document) {
    Array.from(src.styleSheets).forEach(styleSheet => {
        if(styleSheet.ownerNode!==null){
            dest.head.appendChild(styleSheet.ownerNode.cloneNode(true))
        }
    })
    // THIS NEXT RULE IS NEEDED BUT DOESNT WORK
    // Array.from(src.fonts).forEach(font => dest.fonts.add(font))
}