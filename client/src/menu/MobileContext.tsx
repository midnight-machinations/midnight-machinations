import React, { createContext, useEffect, useState } from "react";

const MobileContext = createContext<boolean | undefined>(undefined);
export { MobileContext };

const MOBILE_MAX_WIDTH_PX = 600;

export default function MobileContextProvider(props: { children: React.ReactNode }): React.ReactElement {
    const [mobile, setMobile] = useState<boolean>(false);

    useEffect(() => {
        const onResize = () => {setMobile(window.innerWidth <= MOBILE_MAX_WIDTH_PX)}
        onResize();

        window.addEventListener("resize", onResize);
        return () => window.removeEventListener("resize", onResize);
    }, []);

    return <MobileContext.Provider value={mobile}>
        {props.children}
    </MobileContext.Provider>;
}