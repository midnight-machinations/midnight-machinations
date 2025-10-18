import { ReactElement, useEffect, useState } from "react";
import { useLobbyOrGameState } from "../useHooks";
import React from "react";
import translate from "../../game/lang";


export default function RandomSeedSelector(props: Readonly<{
    disabled?: boolean,
    onChange?: (randomSeed: number | null) => void
}>): ReactElement {
    const randomSeed = useLobbyOrGameState(state => state.randomSeed, ["randomSeed"])??null;

    const [localSeed, setLocalSeed] = useState<number | null>(randomSeed);

    useEffect(() => {
        setLocalSeed(randomSeed);
    }, [randomSeed]);

    const onChange = (seed: number | null) => {
        if (props.onChange) {
            props.onChange(seed);
        }
    }

    return <div className="chat-menu-colors selector-section">
        <h2>{translate("wiki.article.standard.randomSeed.title")}</h2>
        {props.disabled
            ? (randomSeed??translate("none"))
            : <input
                disabled={props.disabled ?? false}
                type="text"
                value={localSeed===null ? "" : localSeed}
                onChange={(e)=>{
                    if(e.target.value === "") {
                        setLocalSeed(null);
                        onChange(null);
                        return;
                    }else{
                        const value = Number(e.target.value);
                        setLocalSeed(isNaN(value) ? null : value);
                        onChange(isNaN(value) ? null : value);
                    }
                }}
            /> 
        }
    </div>
}