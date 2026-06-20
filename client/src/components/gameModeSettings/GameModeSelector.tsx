import React, { ReactElement, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { deleteGameModes, loadGameModesParsed, saveGameModes } from "../../game/localStorage";
import { AnchorControllerContext } from "../../menu/Anchor";
import { CopyButton, PasteButton } from "../../components/ClipboardButtons";
import Icon from "../Icon";
import { Button } from "../Button";
import { DragAndDrop } from "../DragAndDrop";
import { GameModeContext } from "./GameModesEditor";
import translate from "../../game/lang";
import "./gameModeSelector.css"
import parseFromJson, { getLatestFormat } from "./gameMode/migrations";
import { GameMode, GameModeData, GameModeStorage } from "./gameMode";
import { isFailure, parseJsonObject } from "./gameMode/parse";
import Select from "../Select";
import StyledText from "../StyledText";
import { strictDeepEqual } from "../useHooks";
import FlushInput from "../FlushInput";

type GameModeLocation = {
    name: string,
    players: number
}

export function GameModeSelector(props: Readonly<{
    disabled?: boolean,
    loadGameMode: (gameMode: GameModeData) => void,
}>): ReactElement {
    const [gameModeParseResult, setGameModeParseResult] = useState(loadGameModesParsed());

    return <section className="chat-menu-colors selector-section">
        {isFailure(gameModeParseResult)
            ? <div>
                <div>
                    {translate("outdatedGameModesSaveData")}
                    <br />
                    <code>{gameModeParseResult.toString()}</code>
                </div>
                <Button onClick={() => {
                    deleteGameModes();
                    setGameModeParseResult(loadGameModesParsed());
                }}>
                    <Icon>delete</Icon>{translate("deleteOutdatedGameModeSaveData")}
                </Button>
            </div> : <GameModeSelectorPanel {...props} 
                gameModeStorage={gameModeParseResult.value}
                reloadGameModeStorage={() => setGameModeParseResult(loadGameModesParsed())}
            />
        }
    </section>
}

function GameModeSelectorPanel(props: Readonly<{
    gameModeStorage: GameModeStorage,
    disabled?: boolean,
    reloadGameModeStorage: () => void,
    loadGameMode: (gameMode: GameModeData) => void,
}>): ReactElement {
    const [gameModeNameField, setGameModeNameField] = useState<string>("");
    const [gameModeLocation, setGameModeLocation] = useState<GameModeLocation | null>(null);
    const {roleList, phaseTimes, enabledRoles, modifierSettings} = useContext(GameModeContext);
    const anchorController = useContext(AnchorControllerContext)!;

    const validateName = (name: string) => {
        return name.length < 100 && name.length !== 0
    }

    // Caller must ensure location is valid
    const loadGameMode = (location: GameModeLocation) => {
        if (props.disabled) return false;

        const gameMode = props.gameModeStorage.gameModes.find(gameMode => gameMode.name === location.name)!;

        setGameModeNameField(gameMode.name)
        setGameModeLocation(location);
        props.loadGameMode(gameMode.data[location.players]);

        return true;
    }
    
    const saveGameMode = useCallback((name: string) => {
        if(props.disabled) return;
        if(roleList.length === 0) return "noRoles";

        const newGameModeStorage: GameModeStorage = JSON.parse(JSON.stringify(props.gameModeStorage));

        const gameMode = newGameModeStorage.gameModes.find(gameMode => gameMode.name === name);

        if (gameMode === undefined) {
            if (validateName(name)) {
                newGameModeStorage.gameModes.push({
                    name,
                    data: { [roleList.length]: { enabledRoles, phaseTimes, roleList, modifierSettings } }
                })
            } else {
                return "invalidName";
            }
        } else {
            if (Object.keys(gameMode.data).includes("" + roleList.length) && !window.confirm(translate("confirmOverwrite"))) {
                return "didNotConfirm";
            }

            gameMode.data[roleList.length] = {
                roleList,
                phaseTimes,
                enabledRoles,
                modifierSettings
            }
        }

        saveGameModes(newGameModeStorage);
        props.reloadGameModeStorage();
        loadGameMode({
            name: name,
            players: roleList.length
        });
        return "success";
    }, [enabledRoles, props, phaseTimes, roleList, modifierSettings]);

    useEffect(() => {
        const listener = (e: KeyboardEvent) => {
            if (props.disabled !== true && e.ctrlKey && e.key === 's') {
                e.preventDefault();

                const result = saveGameMode(gameModeNameField);

                if (result !== "success") {
                    anchorController.pushErrorCard({
                        title: translate("notification.saveGameMode.failure"), 
                        body: translate("notification.saveGameMode.failure." + result)
                    });
                }
            }
        }
        document.addEventListener('keydown', listener);
        return () => document.removeEventListener('keydown', listener);
    }, [gameModeNameField, anchorController, saveGameMode, props.disabled]);

    useEffect(() => {
        const experimental = props.gameModeStorage.gameModes.find(gameMode => gameMode.name === "Experimental");
        if (experimental) {
            let players; 
            if ("15" in experimental.data) {
                players = 15;
            } else if ("14" in experimental.data) {
                players = 14;
            } else {
                players = Object.keys(experimental.data).map(index => Number.parseInt(index))[0];
            }
            loadGameMode({
                name: "Experimental",
                players
            });
        }
    }, []);

    // Caller must ensure location is valid
    const deleteGameMode = (location: GameModeLocation) => {
        if(props.disabled) return false;
        if(!window.confirm(translate("confirmDelete"))) return false;
        
        const newGameModeStorage: GameModeStorage = JSON.parse(JSON.stringify(props.gameModeStorage));

        const gameModeIndex = newGameModeStorage.gameModes.findIndex(gameMode => gameMode.name === location.name);
        const gameMode = newGameModeStorage.gameModes[gameModeIndex];

        delete gameMode.data[location.players];

        if (Object.keys(gameMode.data).length === 0) {
            newGameModeStorage.gameModes.splice(gameModeIndex, 1);
        }

        saveGameModes(newGameModeStorage);
        props.reloadGameModeStorage();
        return true;
    }

    const verbose = false;

    const shareableGameModeJsonString = JSON.stringify({
        format: getLatestFormat("ShareableGameMode"),
        name: gameModeNameField === "" ? "Unnamed Game Mode" : gameModeNameField,
        roleList,
        phaseTimes,
        enabledRoles,
        modifierSettings
    });

    const shareableGameModeURL = new URL(window.location.href);
    shareableGameModeURL.pathname = "/gameMode"
    shareableGameModeURL.searchParams.set("mode", shareableGameModeJsonString)
    

    return <div className="save-menu">
        {!props.disabled && <GameModeSelectorSelect 
            gameModeStorage={props.gameModeStorage} 
            reloadGameModeStorage={props.reloadGameModeStorage} 
            loadGameMode={loadGameMode}
            gameModeLocation={gameModeLocation}
        />}
        {!props.disabled && <FlushInput 
            value={gameModeNameField}
            setValue={setGameModeNameField}
            onConfirm={setGameModeNameField}
        />}
        {!props.disabled && <div className="vertical-line-separator" />}
        <div>
            {!props.disabled && <Button 
                className="flush" 
                onClick={() => saveGameMode(gameModeNameField)}
                pressedChildren={result => <Icon>{result === "success" ? "done" : "warning"}</Icon>}
                pressedText={result => {
                    if (result === "success") {
                        return translate("notification.saveGameMode.success");
                    } else {
                        return translate("notification.saveGameMode.failure." + result)
                    }
                }}
            >
                <Icon>save</Icon>
            </Button>}
            {!props.disabled && gameModeLocation && <Button className="flush" onClick={() => {
                props.reloadGameModeStorage();
                loadGameMode(gameModeLocation)
            }}>
                <Icon>refresh</Icon>{verbose ? <> {translate("refresh")}</> : undefined}
            </Button>}
            <CopyButton className="flush" text={shareableGameModeJsonString}>
                {verbose ? <><Icon>content_copy</Icon> {translate("copyToClipboard")}</> : undefined}
                </CopyButton>
            {!props.disabled && <PasteButton 
                className="flush"
                onClipboardRead={text => {
                    const json = parseJsonObject(text);
                    if (json === null) {
                        return "invalidData";
                    }
                    const parsedGameMode = parseFromJson("ShareableGameMode", json);
                    if (parsedGameMode.type === "success") {
                        if (parsedGameMode.value.name !== undefined) {
                            setGameModeNameField(parsedGameMode.value.name);
                        }
                        props.loadGameMode({
                            roleList: parsedGameMode.value.roleList,
                            phaseTimes: parsedGameMode.value.phaseTimes,
                            enabledRoles: parsedGameMode.value.enabledRoles,
                            modifierSettings: parsedGameMode.value.modifierSettings
                        })
                    } else {
                        anchorController.pushErrorCard({
                            title: translate("outdatedGameModeSaveData"), 
                            body: translate("outdatedGameModeSaveData.details") + parsedGameMode.toString()
                        })
                        return "invalidData";
                    }
                }}
                failureText={() => translate("notification.importGameMode.failure")}
            >{verbose ? <><Icon>content_paste_go</Icon> {translate("importFromClipboard")}</> : <Icon>content_paste_go</Icon>}</PasteButton>}
            <CopyButton className="flush" text={shareableGameModeURL.toString()}>
                <Icon>share</Icon>{verbose ? <> {translate("copyToClipboard")}</> : undefined}
            </CopyButton>
        </div>
    </div>
}

function GameModeSelectorSelect(props: Readonly<{
    gameModeStorage: GameModeStorage,
    reloadGameModeStorage: () => void,
    loadGameMode: (gameMode: GameModeLocation) => void,
    gameModeLocation: GameModeLocation | null
}>): ReactElement {
    const [star, setStar] = useState(false);

    const gameModeData = useMemo(() => {
        if (props.gameModeLocation) {
            const gameMode = props.gameModeStorage.gameModes.find(gm => gm.name === props.gameModeLocation!.name);
            return gameMode ? gameMode.data[props.gameModeLocation.players] : null;
        }
        return null;
    }, [props.gameModeLocation]);

    const gameModeContext = useContext(GameModeContext);

    useEffect(() => {
        console.log(gameModeData, gameModeContext);
        setStar(!strictDeepEqual<GameModeData | null>(gameModeData, gameModeContext));
    }, [gameModeData, gameModeContext]);

    const keyMap = useMemo(() => {
        const map = new Map<string, GameModeLocation>();

        for (const gameMode of props.gameModeStorage.gameModes) {
            for (const [number, _] of Object.entries(gameMode.data)) {
                const players = Number.parseInt(number);

                map.set(
                    gameMode.name + ":" + players,
                    {
                        name: gameMode.name,
                        players,
                    }
                );
            }
        }
        

        return map;
    }, [props.gameModeStorage]);

    const optionsSearch = useMemo(() => {
        const options = new Map<string, [React.ReactNode, string]>();

        if (props.gameModeLocation === null) {
            options.set("Custom", [
                <StyledText key="custom" noLinks={true}>
                    Custom
                </StyledText>,
                "Custom"
            ]);
        }

        for (const gameMode of props.gameModeStorage.gameModes) {
            for (const [number, _] of Object.entries(gameMode.data)) {
                const players = Number.parseInt(number);
                let addStar = false;
                if (props.gameModeLocation !== null && props.gameModeLocation.name === gameMode.name && props.gameModeLocation.players === players) {
                    addStar = star;
                }

                options.set(
                    gameMode.name + ":" + players,
                    [
                        <StyledText key={gameMode.name + ":" + players} noLinks={true}>
                            {`${gameMode.name}: ${players}${addStar ? "*" : ""}`}
                        </StyledText>,
                        gameMode.name + " " + players
                    ]
                );
            }
        }

        return options;
    }, [props.gameModeStorage, star, props.gameModeLocation]);

    const selectedValue = useMemo(() => {
        if (props.gameModeLocation === null) {
            return "Custom" as const;
        } else {
            return props.gameModeLocation.name + ":" + props.gameModeLocation.players;
        }
    }, [props.gameModeLocation, props.gameModeStorage])

    return <Select className="brand" value={selectedValue} optionsSearch={optionsSearch} onChange={value => {
        if (value === "Custom") {
            props.reloadGameModeStorage();
            return;
        }
        props.loadGameMode(keyMap.get(value)!);
        props.reloadGameModeStorage();
    }} />
}

function GameModeLabel(props: Readonly<{ 
    gameMode: GameMode,
    modifiable: boolean,
    gameModeStorage: GameModeStorage,
    loadGameMode: (location: GameModeLocation) => boolean, 
    deleteGameMode: (location: GameModeLocation) => boolean,
    dragHandleProps?: React.HTMLAttributes<HTMLDivElement>
}>): ReactElement {
    if (Object.keys(props.gameMode.data).length === 1) {
        return <GameModeSingleLabel 
            location={{ name: props.gameMode.name, players: parseInt(Object.keys(props.gameMode.data)[0]) }}
            modifiable={props.modifiable}
            gameModeStorage={props.gameModeStorage}
            loadGameMode={props.loadGameMode}
            deleteGameMode={props.deleteGameMode}
            dragHandleProps={props.dragHandleProps}
        />
    } else {
        return <GameModeFolderLabel
            gameModeName={props.gameMode.name}
            modifiable={props.modifiable}
            gameModeStorage={props.gameModeStorage}
            loadGameMode={props.loadGameMode}
            deleteGameMode={props.deleteGameMode}
            dragHandleProps={props.dragHandleProps}
        />
    }
}

function GameModeFolderLabel(props: {
    gameModeName: string,
    modifiable: boolean,
    gameModeStorage: GameModeStorage,
    loadGameMode: (location: GameModeLocation) => boolean,
    deleteGameMode: (location: GameModeLocation) => boolean,
    dragHandleProps?: React.HTMLAttributes<HTMLDivElement>
}): ReactElement {
    const [expanded, setExpanded] = useState<boolean>(false);

    useEffect(() => {
        setExpanded(false)
    }, [props.gameModeName])
    
    const gameMode = props.gameModeStorage.gameModes.find(gameMode => gameMode.name === props.gameModeName)!

    return <div className="game-mode-label">
        {props.modifiable && <div {...props.dragHandleProps}><Icon>drag_indicator</Icon></div>}
        <div className="game-mode-folder">
            <Button onClick={() => setExpanded(!expanded)} className="game-mode-folder-header">
                <span className="game-mode-name">{props.gameModeName}</span>
                <Icon>{expanded ? "expand_less" : "expand_more"}</Icon>
            </Button>
            {expanded && <div className="game-mode-folder-content">
                {Object.keys(gameMode.data).map(key => <GameModeSingleLabel
                    location={{ name: props.gameModeName, players: parseInt(key) }}
                    gameModeStorage={props.gameModeStorage}
                    modifiable={props.modifiable}
                    draggable={false}
                    loadGameMode={props.loadGameMode}
                    deleteGameMode={props.deleteGameMode}
                />)}
            </div>}
        </div>
    </div>
}

function GameModeSingleLabel(props: { 
    location: GameModeLocation, 
    gameModeStorage: GameModeStorage,
    loadGameMode: (location: GameModeLocation) => boolean, 
    dragHandleProps?: React.HTMLAttributes<HTMLDivElement>
} & (
    {
        modifiable: true,
        draggable?: boolean,
        deleteGameMode: (location: GameModeLocation) => boolean
    } | {
        modifiable?: false
    }
)): ReactElement {
    return <div className="game-mode-label">
        {props.modifiable && (props.draggable ?? true) && <Icon {...props.dragHandleProps}>drag_indicator</Icon>}
        <span className="game-mode-name">{props.location.name}: {props.location.players}</span>
        <div className="game-mode-label-buttons">
            <Button 
                onClick={() => props.loadGameMode(props.location)}
                pressedChildren={result => <Icon>{result ? "done" : "warning"}</Icon>}
            ><Icon>{props.modifiable ? "edit" : "launch"}</Icon></Button>
            {props.modifiable && <Button 
                onClick={() => props.deleteGameMode(props.location)}
                pressedChildren={result => <Icon>{result ? "done" : "warning"}</Icon>}
            ><Icon>delete</Icon></Button>}
        </div>
    </div>
}