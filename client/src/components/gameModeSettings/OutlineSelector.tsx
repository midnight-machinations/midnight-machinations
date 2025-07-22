import React, { ReactElement, useCallback, useContext, useMemo, useRef, useState } from "react";
import "./outlineSelector.css";
import translate from "../../game/lang";
import { getAllRoles, getRolesFromRoleSet, ROLE_SETS, RoleList, RoleOrRoleSet, RoleOutline, simplifyRoleOutline, translateRoleOutline, translateRoleOrRoleSet} from "../../game/roleListState.d";
import { Role } from "../../game/roleState.d";
import Icon from "../Icon";
import { DragAndDrop } from "../DragAndDrop";
import { GameModeContext } from "./GameModesEditor";
import Select, { dropdownPlacementFunction, SelectOptionsSearch } from "../Select";
import StyledText from "../StyledText";
import { Button, RawButton } from "../Button";
import { useLobbyOrGameState, useLobbyState } from "../useHooks";
import { Conclusion, CONCLUSIONS, INSIDER_GROUPS, InsiderGroup, PlayerClientType, PlayerIndex, translateConclusion, translateWinCondition } from "../../game/gameState.d";
import Popover from "../Popover";

type RoleOutlineSelectorProps = {
    roleOutline: RoleOutline,
    onChange: (value: RoleOutline) => void,
    disabled?: boolean,
}

export default function RoleOutlineSelector(props: RoleOutlineSelectorProps): ReactElement {
    const handleAddUnion = () => {
        props.onChange([...props.roleOutline, { roleSet: "any" }]);
    }

    return <div className="role-picker">
        {props.roleOutline.map((option, index) => {
            let roleOrRoleSet: RoleOrRoleSet;

            if ("role" in option) {
                roleOrRoleSet = {
                    type: "role",
                    role: option.role
                }
            } else {
                roleOrRoleSet = {
                    type: "roleSet",
                    roleSet: option.roleSet
                }
            }

            return (
                <div key={index} className="role-picker-option">
                    <PlayerPoolSelectorLabel
                        disabled={props.disabled}
                        playerPool={option.playerPool}
                        onChange={pool => {
                            const options = [...props.roleOutline];

                            if(pool === undefined && "playerPool" in options[index]) {
                                delete options[index].playerPool;
                            } else {
                                options[index].playerPool = pool;
                            }

                            props.onChange(options)
                        }}
                    />
                    <InsiderGroupSelectorLabel
                        disabled={props.disabled}
                        insiderGroups={option.insiderGroups}
                        onChange={groups => {
                            const options = [...props.roleOutline];

                            if(groups === undefined && "insiderGroups" in options[index]) {
                                delete options[index].insiderGroups;
                            } else {
                                options[index].insiderGroups = groups;
                            }

                            props.onChange(options)
                        }}
                    />
                    <ConclusionsSelectorLabel
                        disabled={props.disabled}
                        conclusions={option.winIfAny}
                        onChange={concs => {
                            const options = [...props.roleOutline];

                            if(concs === undefined && "winIfAny" in options[index]) {
                                delete options[index].winIfAny;
                            } else {
                                options[index].winIfAny = concs;
                            }

                            props.onChange(options)
                        }}
                    />
                    <RoleOrRoleSetSelector
                        disabled={props.disabled}
                        roleOrRoleSet={roleOrRoleSet}
                        onChange={(value) => {
                            let options = [...props.roleOutline];

                            let old = {...options[index]};

                            switch (value.type) {
                                case "role":
                                    options[index] = {
                                        role: value.role
                                    }
                                    break;
                                case "roleSet":
                                    options[index] = {
                                        roleSet: value.roleSet
                                    }
                                    break;
                            }
                            
                            if("winIfAny" in old)
                                options[index].winIfAny = old.winIfAny;
                            if("insiderGroups" in old)
                                options[index].insiderGroups = old.insiderGroups;

                            props.onChange(options);
                        }}
                    />
                    <Button
                        disabled={props.disabled}
                        onClick={() => {
                            let options = [...props.roleOutline];
                            options.splice(index, 1);
                            if(options.length === 0) {
                                props.onChange([{ roleSet: "any" }]);
                                return
                            }
                            props.onChange(options);
                        }}
                    ><Icon size="tiny">remove</Icon></Button>
                </div>
            )
        })}
        <Button
            disabled={props.disabled}
            onClick={() => {
                handleAddUnion();
            }}
        ><Icon size="tiny">add</Icon></Button>
    </div>
}

function ConclusionsSelector(props: Readonly<{
    disabled?: boolean,
    conclusions?: Conclusion[],
    onChange: (newSet?: Conclusion[]) => void,
}>): ReactElement {
    if (props.conclusions === undefined) {
        return <div className="conclusions-selector">
            <Button
                onClick={() => props.onChange(["town"])}
            >
                {translate("setNotDefault")}
            </Button>
        </div>
    }

    const conclusions = props.conclusions;
    const conclusionsNotChosen = CONCLUSIONS.filter(conc => !conclusions.includes(conc));

    const optionsSearch = new Map<Conclusion, [ReactElement, string]>(CONCLUSIONS.map(conclusion => [
        conclusion, [
            <StyledText noLinks={true}>{translateConclusion(conclusion)}</StyledText>, 
            translateConclusion(conclusion)
        ]
    ]));

    return <div className="conclusions-selector">
        <div className="role-picker">
            {conclusions.map((option, index) => {
                return (
                    <div key={index} className="role-picker-option">
                        <Select 
                            className="role-outline-option-selector"
                            disabled={props.disabled}
                            value={option}
                            onChange={value => {
                                const options = [...conclusions];
                                options[index] = value;
                                props.onChange(options);
                            }}
                            optionsSearch={optionsSearch}
                        />
                        <Button
                            disabled={props.disabled}
                            onClick={() => {
                                const options = [...conclusions];
                                options.splice(index, 1);
                                props.onChange(options);
                            }}
                        ><Icon size="tiny">remove</Icon></Button>
                    </div>
                )
            })}
            {conclusionsNotChosen.length !== 0 && <Button
                disabled={props.disabled}
                onClick={() => props.onChange([...conclusions, conclusionsNotChosen[0]])}
            ><Icon size="tiny">add</Icon></Button>}
        </div>
        <Button
            disabled={props.disabled}
            onClick={() => props.onChange()}
        >
            {translate("setDefault")}
        </Button>
    </div>
}

function ConclusionsSelectorLabel(props: Readonly<{
    disabled?: boolean,
    conclusions?: Conclusion[],
    onChange: (value?: Conclusion[]) => void,
}>): ReactElement {
    const ref = useRef<HTMLButtonElement>(null);

    const [popupOpen, setPopupOpen] = useState<boolean>(false);

    const buttonDisplay = useMemo(() => {
        if (props.conclusions === undefined) {
            return <Icon>emoji_events</Icon>
        } else {
            return <StyledText noLinks={true}>
                {translateWinCondition({ type: "gameConclusionReached", winIfAny: props.conclusions })}
            </StyledText>
        }
    }, [props.conclusions])
    
    return <>
        <RawButton
            ref={ref}
            disabled={props.disabled}
            onClick={() => setPopupOpen(open => !open)}
        >
            {buttonDisplay}
        </RawButton>
        <Popover
            open={popupOpen}
            setOpenOrClosed={setPopupOpen}
            anchorForPositionRef={ref}
            onRender={dropdownPlacementFunction}
        >
            <ConclusionsSelector
                disabled={props.disabled}
                conclusions={props.conclusions}
                onChange={props.onChange}
            />
        </Popover>
    </>
}

function InsiderGroupSelector(props: Readonly<{
    disabled?: boolean,
    insiderGroups?: InsiderGroup[],
    onChange: (newSet?: InsiderGroup[]) => void,
}>): ReactElement {
    if (props.insiderGroups === undefined) {
        return <div className="conclusions-selector">
            <Button
                onClick={() => props.onChange(["mafia"])}
            >
                {translate("setNotDefault")}
            </Button>
        </div>
    }

    const insiderGroups = props.insiderGroups;
    const insiderGroupsNotChosen = INSIDER_GROUPS.filter(conc => !insiderGroups.includes(conc));

    const optionsSearch = new Map<InsiderGroup, [ReactElement, string]>(INSIDER_GROUPS.map(insiderGroup => [
        insiderGroup, [
            <StyledText noLinks={true}>{translate(`chatGroup.${insiderGroup}.name`)}</StyledText>,
            translate(`chatGroup.${insiderGroup}.name`)
        ]
    ]));

    return <div className="conclusions-selector">
        <div className="role-picker">
            {insiderGroups.map((option, index) => {
                return (
                    <div key={index} className="role-picker-option">
                        <Select 
                            className="role-outline-option-selector"
                            disabled={props.disabled}
                            value={option}
                            onChange={value => {
                                const options = [...insiderGroups];
                                options[index] = value;
                                props.onChange(options);
                            }}
                            optionsSearch={optionsSearch}
                        />
                        <button
                            disabled={props.disabled}
                            onClick={() => {
                                const options = [...insiderGroups];
                                options.splice(index, 1);
                                props.onChange(options);
                            }}
                        ><Icon size="tiny">remove</Icon></button>
                    </div>
                )
            })}
            {insiderGroupsNotChosen.length !== 0 && <button
                disabled={props.disabled}
                onClick={() => props.onChange([...insiderGroups, insiderGroupsNotChosen[0]])}
            ><Icon size="tiny">add</Icon></button>}
        </div>
        <Button
            disabled={props.disabled}
            onClick={() => props.onChange()}
        >
            {translate("setDefault")}
        </Button>
    </div>
}

function InsiderGroupSelectorLabel(props: Readonly<{
    disabled?: boolean,
    insiderGroups?: InsiderGroup[],
    onChange: (value?: InsiderGroup[]) => void,
}>): ReactElement {
    const ref = useRef<HTMLButtonElement>(null);

    const [popupOpen, setPopupOpen] = useState<boolean>(false);

    const buttonDisplay = useMemo(() => {
        if (props.insiderGroups === undefined) {
            return <Icon>chat_bubble_outline</Icon>
        } else if (props.insiderGroups.length === 0) {
            return <StyledText noLinks={true}>
                {translate("chatGroup.all.icon")}
            </StyledText>
        } else {
            return <StyledText noLinks={true}>
                {props.insiderGroups.map(g => translate(`chatGroup.${g}.icon`)).join()}
            </StyledText>
        }
    }, [props.insiderGroups])
    
    return <>
        <RawButton
            ref={ref}
            disabled={props.disabled}
            onClick={() => setPopupOpen(open => !open)}
        >
            {buttonDisplay}
        </RawButton>
        <Popover
            open={popupOpen}
            setOpenOrClosed={setPopupOpen}
            anchorForPositionRef={ref}
            onRender={dropdownPlacementFunction}
        >
            <InsiderGroupSelector
                disabled={props.disabled}
                insiderGroups={props.insiderGroups}
                onChange={props.onChange}
            />
        </Popover>
    </>
}

function PlayerPoolSelector(props: Readonly<{
    disabled?: boolean,
    playerPool?: PlayerIndex[],
    onChange: (newSet?: PlayerIndex[]) => void,
}>): ReactElement {
    const playerNames = useLobbyState(
        state => 
            state.players.list
                .filter(([_id, client]) => client.clientType.type === "player")
                .map(([_id, player]) => (player.clientType as PlayerClientType).name),
        ["lobbyClients"]
    )!;

    if (props.playerPool === undefined) {
        if (playerNames.length > 0) {
            return <div className="conclusions-selector">
                <Button
                    onClick={() => props.onChange([0])}
                >
                    {translate("setNotDefault")}
                </Button>
            </div>
        } else {
            // This shouldn't be possible anyway, but just in case.
            return <div className="conclusions-selector">
                <StyledText noLinks={true}>{translate("noPlayers")}</StyledText>
            </div>
        }
    }

    const playerPool = props.playerPool;
    const playersNotChosen = playerNames.map((_, index)=>index).filter(index => !playerPool.includes(index));

    const optionsSearch = new Map<number, [ReactElement, string]>(playerNames.map((name, index) => [
        index, [<StyledText noLinks={true}>{name}</StyledText>, name]
    ]));

    return <div className="conclusions-selector">
        <div className="role-picker">
            {playerPool.map((id, index) => {
                return (
                    <div key={id} className="role-picker-option">
                        <Select
                            className="role-outline-option-selector"
                            disabled={props.disabled}
                            value={id}
                            onChange={value => {
                                const options = [...playerPool];
                                options[index] = value;
                                props.onChange(options);
                            }}
                            optionsSearch={optionsSearch}
                        />
                        <Button
                            disabled={props.disabled}
                            onClick={() => {
                                const options = [...playerPool];
                                options.splice(index, 1);
                                props.onChange(options);
                            }}
                        ><Icon size="tiny">remove</Icon></Button>
                    </div>
                )
            })}
            {playersNotChosen.length !== 0 && <Button
                disabled={props.disabled}
                onClick={() => props.onChange([...playerPool, playersNotChosen[0]])}
            ><Icon size="tiny">add</Icon></Button>}
        </div>
        <Button
            disabled={props.disabled}
            onClick={() => props.onChange()}
        >
            {translate("setDefault")}
        </Button>
    </div>
}

function PlayerPoolSelectorLabel(props: Readonly<{
    disabled?: boolean,
    playerPool?: PlayerIndex[],
    onChange: (value?: PlayerIndex[]) => void,
}>): ReactElement {
    const ref = useRef<HTMLButtonElement>(null);

    const [popupOpen, setPopupOpen] = useState<boolean>(false);

    const buttonDisplay = useMemo(() => {
        if (props.playerPool === undefined) {
            return <Icon>diversity_1</Icon>
        } else if (props.playerPool.length === 0) {
            return <Icon>person_off</Icon>
        } else {
            return <Icon>assignment_ind</Icon>
        }
    }, [props.playerPool]);

    return <>
        <RawButton
            ref={ref}
            onClick={() => setPopupOpen(open => !open)}
        >
            {buttonDisplay}
        </RawButton>
        <Popover
            open={popupOpen}
            setOpenOrClosed={setPopupOpen}
            anchorForPositionRef={ref}
            onRender={dropdownPlacementFunction}
        >
            <PlayerPoolSelector
                disabled={props.disabled}
                playerPool={props.playerPool}
                onChange={props.onChange}
            />
        </Popover>
    </>
}

export function RoleOrRoleSetSelector(props: Readonly<{
    disabled?: boolean,
    roleOrRoleSet: RoleOrRoleSet,
    onChange: (value: RoleOrRoleSet) => void,
}>): ReactElement {
    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;

    const isRoleEnabled = useCallback((role: Role) => {
        return enabledRoles.includes(role)
    }, [enabledRoles])

    const optionsSearch: SelectOptionsSearch<string> = new Map();

    ROLE_SETS.forEach((roleSet) => {
        optionsSearch.set(JSON.stringify({type: "roleSet", roleSet: roleSet}), [
            <StyledText
                key={0}
                noLinks={!props.disabled}
                className={getRolesFromRoleSet(roleSet).every(role => !isRoleEnabled(role)) ? "keyword-disabled" : ""}
            >
                {translateRoleOrRoleSet({type: "roleSet", roleSet: roleSet})}
            </StyledText>, 
            translateRoleOrRoleSet({type: "roleSet", roleSet: roleSet})]
        );
    });
    
    getAllRoles().forEach((role) => {
        optionsSearch.set(JSON.stringify({type: "role", role: role}), [
            <StyledText
                key={0}
                noLinks={!props.disabled}
                className={!isRoleEnabled(role) ? "keyword-disabled" : ""}
            >
                {translateRoleOrRoleSet({type: "role", role})}
            </StyledText>,
            translateRoleOrRoleSet({type: "role", role})
        ]);
    });

    return <Select
        className="role-outline-option-selector"
        disabled={props.disabled}
        value={JSON.stringify(props.roleOrRoleSet)}
        onChange={(value) => {
            props.onChange(
                value === "any" ? "any" : JSON.parse(value)
            );
        }}
        optionsSearch={optionsSearch}
    />
}

export function OutlineListSelector(props: Readonly<{
    disabled?: boolean,
    onChangeRolePicker: (value: RoleOutline, index: number) => void,
    onAddNewOutline?: (() => void),
    onRemoveOutline?: ((index: number) => void),
    setRoleList: (newRoleList: RoleList) => void,
}>) {
    const {roleList} = useContext(GameModeContext);

    const simplify = () => {
        props.setRoleList(roleList.map(simplifyRoleOutline));
    }

    return <section className="graveyard-menu-colors selector-section">
        <h2>{translate("menu.lobby.roleList")}: {roleList.length}</h2>
        {(props.disabled !== true) && <Button onClick={simplify}>
            <Icon>filter_list</Icon> {translate("simplify")}
        </Button>}
        <div className="role-list-setter-list">
            <DragAndDrop 
                items={structuredClone(roleList)}
                onDragEnd={props.setRoleList}
                disabled={props.disabled}
                render={(outline, index) => {
                    return <div key={index} className="role-list-setter-outline-div">
                        {props.disabled === true || <Icon>drag_indicator</Icon>}
                        {props.disabled === true
                            ? <div className="placard">
                                <StyledText>
                                    {translateRoleOutline(outline)}
                                </StyledText>
                            </div>
                            : <RoleOutlineSelector
                                disabled={props.disabled}
                                roleOutline={outline}
                                onChange={(value: RoleOutline) => {props.onChangeRolePicker(value, index);}}
                                key={index}
                            />
                        }
                        {props.onRemoveOutline &&
                            <button disabled={props.disabled} onClick={() => {
                                if(props.onRemoveOutline)
                                    props.onRemoveOutline(index)
                        }}><Icon>delete</Icon></button>}
                    </div>
                }}
            />
            <div className="role-list-setter-outline-div role-list-setter-add-button-div">
                {props.onAddNewOutline ? 
                    <button disabled={props.disabled} onClick={props.onAddNewOutline}>
                        <Icon>add</Icon>
                    </button> : null}
            </div>
        </div>
    </section>
}

