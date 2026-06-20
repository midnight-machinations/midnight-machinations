import React, { useCallback, useEffect, useMemo, useRef } from "react";
import { RawButton } from "./Button";
import "./select.css";
import Icon from "./Icon";
import Popover from "./Popover";
import translate from "../game/lang";

export type SelectOptionsNoSearch<K extends { toString(): string}> = Map<K, React.ReactNode>;
export type SelectOptionsSearch<K extends { toString(): string}> = Map<K, [React.ReactNode, string]>;

export default function Select<K extends { toString(): string}>(props: Readonly<{
    value: K,
    disabled?: boolean,
    className?: string,
    onChange?: (value: K)=>void,
    noCloseOnKeyboardSelect?: boolean
    optionsSearch: SelectOptionsSearch<K>,
    hideArrow?: true
}>) {
    const optionsSearch: SelectOptionsSearch<K> = useMemo(() => props.optionsSearch, [props]);

    const [open, setOpen] = React.useState(false);
    const [searchString, setSearchString] = React.useState("");

    const [indexSelected, setIndexSelected] = React.useState<number>(0);
    
    const allSearchResults = useMemo(() => {
        if (searchString === "") {
            return [...optionsSearch.keys()];
        }

        const searchResults = [...optionsSearch.keys()].filter((key) => {
            for(const search of searchString.split(" ")) {
                
                const val = optionsSearch.get(key);
                if(val === undefined) {return false}
                if(!val[1].toLowerCase().includes(search.toLowerCase())){
                    return false;
                }
            }
            return true;
        });

        //sort by length and take the first. If you type "witch" we don't want "syndicate witch"
        searchResults.sort((a, b) => a.toString().length - b.toString().length);
        return searchResults;
    }, [optionsSearch, searchString]);

    const handleOnChange = useCallback((key: K) => {
        setSearchString("");
        if(props.onChange && key !== props.value) {
            props.onChange(key);
        }
    }, [props]);
    const handleSetOpen = useCallback((isOpen: boolean) => {
        setOpen(isOpen);
        setIndexSelected(0);
        setSearchString("");
    }, []);

    const handleKeyInput = (inputKey: string) => {
        switch(inputKey) {
            case "ArrowDown":
                if (open) {
                    setIndexSelected((prev) => Math.min(Math.max(prev + 1, 0), allSearchResults.length - 1));
                } else {
                    handleSetOpen(true);
                }
                break;
            case "ArrowUp":
                if (open) {
                    setIndexSelected((prev) => Math.min(Math.max(prev - 1, 0), allSearchResults.length - 1));
                }
                break;
            case "Escape":
                handleSetOpen(false);
                break;
            case "Enter": {
                if(indexSelected !== undefined && indexSelected >= 0 && indexSelected < allSearchResults.length && allSearchResults[indexSelected] !== undefined) {
                    handleOnChange(allSearchResults[indexSelected]);
                } else if(allSearchResults[0] !== undefined) {
                    handleOnChange(allSearchResults[0]);
                }
                // eslint-disable-next-line no-negated-condition
                if (props.noCloseOnKeyboardSelect !== true) {
                    handleSetOpen(false);
                } else {
                    setIndexSelected(0);
                }

                break;
            }
            case "Backspace":
                setSearchString(searchString.substring(0,searchString.length-1));
                setIndexSelected(0);
                break;
            default:
                if(/^[a-zA-Z0-9- ]$/.test(inputKey)) {
                    setSearchString(searchString+inputKey);
                    setIndexSelected(0);
                }
        }
    }

    const ref = useRef<HTMLButtonElement>(null);

    const value = optionsSearch.get(props.value);
    if(value === undefined) {
        console.error(`Value not found in options ${props.value}`);
    }

    return <>
        <RawButton
            ref={ref}
            disabled={props.disabled}
            onClick={()=>{handleSetOpen(!open)}}
            className={"custom-select "+(props.className?props.className:"")}
            onKeyDown={(e)=>{
                if(props.disabled) return;
                if(e.key === "Enter" && !open) {
                    e.preventDefault();
                    handleSetOpen(true);
                }else if(e.key === "Tab") {
                    handleSetOpen(false);
                }else{
                    e.preventDefault();
                    handleKeyInput(e.key);
                }
            }}
        >
            {props.hideArrow !== true && (open === true ? 
                <Icon>keyboard_arrow_up</Icon> :
                <Icon>keyboard_arrow_down</Icon>)}
            {value !== undefined?value[0]:props.value.toString()}
        </RawButton>
        <Popover className="custom-select-options-popover"
            open={open}
            setOpenOrClosed={handleSetOpen}
            onRender={selectPlacementFunction}
            anchorForPositionRef={ref}
        >
            <div>
                {searchString!==""?<>{translate("menu.ability.icon")}<span>{searchString===""?undefined:searchString.substring(0, 20)}</span></>:""}
                <SelectOptions 
                    options={optionsSearch}
                    allSearchResults={allSearchResults}
                    onChange={(value)=>{
                        if(props.disabled) return;
                        handleSetOpen(false);
                        handleOnChange(value);
                    }}
                    indexSelected={indexSelected}
                />
            </div>
        </Popover>
    </>
}

export function selectPlacementFunction(dropdownElement: HTMLElement, buttonElement: HTMLElement | undefined) {
    if (!buttonElement) return;

    const buttonBounds = buttonElement.getBoundingClientRect();
    dropdownElement.style.width = `${buttonBounds.width}px`;

    dropdownPlacementFunction(dropdownElement, buttonElement);
}

/// Assumes there is only 1 element inside Popover
export function dropdownPlacementFunction(dropdownElement: HTMLElement, buttonElement: HTMLElement | undefined, heightLimitRem: number | null = 25) {
    if (!buttonElement) return;

    const buttonBounds = buttonElement.getBoundingClientRect();
    dropdownElement.style.left = `${buttonBounds.left}px`;

    const spaceAbove = buttonBounds.top;
    const spaceBelow = window.innerHeight - buttonBounds.bottom;

    const oneRem = parseFloat(getComputedStyle(buttonElement).fontSize);

    const maxHeight = heightLimitRem === null ? Infinity : (heightLimitRem - .25) * oneRem;
    const optionsHeight = 1 + .5 * oneRem + (dropdownElement.firstElementChild?.clientHeight ?? Infinity);

    if (spaceAbove > spaceBelow) {
        const newHeight = heightLimitRem === null ? optionsHeight : Math.min(maxHeight, spaceAbove - .25 * oneRem, optionsHeight);
        dropdownElement.style.height = `${newHeight}px`;
        dropdownElement.style.top = `unset`;
        dropdownElement.style.bottom = `${spaceBelow + buttonBounds.height + .25 * oneRem}px`;
    } else {
        const newHeight = heightLimitRem === null ? optionsHeight : Math.min(maxHeight, spaceBelow - .25 * oneRem, optionsHeight);
        dropdownElement.style.height = `${newHeight}px`;
        dropdownElement.style.top = `${spaceAbove + buttonBounds.height + .25 * oneRem}px`;
        dropdownElement.style.bottom = `unset`;
    }

    keepPopoverOnScreen(dropdownElement, buttonElement);
}

export function keepPopoverOnScreen(dropdownElement: HTMLElement, buttonElement?: HTMLElement) {
    const dropdownBounds = dropdownElement.getBoundingClientRect();

    const modifyTop = dropdownElement.style.bottom === 'unset' || dropdownElement.style.bottom === "";
    const modifyLeft = dropdownElement.style.right === 'unset' || dropdownElement.style.right === "";

    const spaceAbove = dropdownBounds.top;
    const spaceBelow = window.innerHeight - dropdownBounds.bottom;
    const spaceToTheRight = window.innerWidth - dropdownBounds.right;
    const spaceToTheLeft = dropdownBounds.left;

    if (spaceToTheRight < 0) {
        if (modifyLeft) {
            dropdownElement.style.left = `${window.innerWidth - dropdownBounds.width}px`
        } else {
            dropdownElement.style.right = "0px"
        }
    }

    if (spaceToTheLeft < 0) {
        if (modifyLeft) {
            dropdownElement.style.left = "0px"
        } else {
            dropdownElement.style.right = `${dropdownBounds.width}px`
        }
    }

    if (spaceBelow < 0) {
        if (modifyTop) {
            dropdownElement.style.top = `${window.innerHeight - dropdownBounds.height}px`
        } else {
            dropdownElement.style.bottom = "0px"
        }
    }

    if (spaceAbove < 0) {
        if (modifyTop) {
            dropdownElement.style.top = "0px"
        } else {
            dropdownElement.style.bottom = `${spaceBelow + spaceAbove}px`
        }
    }
}

function SelectOptions<K extends { toString(): string}>(props: Readonly<{
    options: SelectOptionsSearch<K>,
    onChange?: (value: K)=>void,
    allSearchResults?: K[],
    indexSelected?: number,
}>) {
    let options: SelectOptionsSearch<K>;
    if (props.allSearchResults === undefined || props.allSearchResults.length === 0) {
        options = props.options;
    } else {
        const ordered: [K, [React.ReactNode, string]][] = props.allSearchResults?.map((value) => {
            const option = props.options.get(value);
            if (option) {
                return [value, option];
            }
            // This shouldn't happen
            return [value, [value.toString() as React.ReactNode, value.toString()]];
        });
        options = ordered ? new Map(ordered) : props.options;
    }

    const current = useRef<HTMLButtonElement | null>(null);

    useEffect(() => {
        if (current.current) {
            current.current.scrollIntoView({ block: "center" });
        }
    }, [props.indexSelected]);

    return <div className="custom-select-options">
        <div>
            {[...options.entries()]
                .map(([key, [value, _]], index) => {
                    return <RawButton
                        className={index === props.indexSelected ? "fauxcus-visible" : ""}
                        key={key.toString()}
                        onClick={()=>{
                            if(props.onChange) {
                                props.onChange(key);
                            }
                        }}
                        ref={index === props.indexSelected ? current : null}
                    >
                        {value}
                    </RawButton>
                })
            }
        </div>
    </div>
}