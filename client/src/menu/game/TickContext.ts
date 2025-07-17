import { createContext, useEffect, useState } from "react";

export type TickData = {count: number, timeDelta: number}
const TickContext = createContext<TickData | undefined>(undefined)
export { TickContext }

export function useTickContext(){
    const [tickData, setTickData] = useState<TickData>({
        count: 0,
        timeDelta: 0
    });

    //start tick
    useEffect(() => {
        const TICK_TIME_DELTA = 1000;
        let tickInterval = setInterval(()=>{
            setTickData(tickData => ({
                count: tickData.count + 1,
                timeDelta: TICK_TIME_DELTA
            }));
        }, TICK_TIME_DELTA);

        return ()=>clearInterval(tickInterval)
    }, []);

    return tickData
}