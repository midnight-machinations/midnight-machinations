import { ReactElement, ReactNode, useEffect, useMemo, useState } from "react";
import "./gameResults.css"
import { PlayerSynopsis, SynopsisCrumb } from "../../game/packet";
import ChatElement, { encodeString } from "../../components/ChatMessage";
import translate from "../../game/lang";
import { Conclusion, translateConclusion, translateWinCondition } from "../../game/gameState.d";
import { RoleList, translateInsiderGroups, translateRoleOutline } from "../../game/roleListState.d";
import StyledText, { getStyledHtmlFromString, KeywordDataMap, TokenData } from "../../components/StyledText";
import GraveComponent from "../../components/grave";
import { loadStreakParsed, saveStreak, Streak } from "../../game/localStorage";
import { createPlayer } from "../../game/gameState";

const DAY_IN_MS = 86_400_000;
const WEEK_IN_MS = DAY_IN_MS * 7;

type StreakProgress = {
    length: number,
    advanced: boolean,
    lastPlayed: number,
    justDied: boolean
}

type StreakWeek = {
    weekNumber: number,
    current: boolean,
    next: boolean
}

export default function GameResults(props: Readonly<{
    playerSynopsis: PlayerSynopsis
    playerNames: string[],
    conclusion: Conclusion,
    roleList: RoleList
}>): ReactElement {
    const [streakProgress] = useState<StreakProgress>(() => getStreakProgress(loadStreakParsed()));

    useEffect(() => {
        saveStreak({
            length: streakProgress.length,
            lastPlayed: streakProgress.lastPlayed
        });
    }, [streakProgress]);

    const players = useMemo(() => {
        return props.playerNames.map((name, index) => createPlayer(name, index))
    }, [props.playerNames])

    const playerNames = useMemo(() => {
        return players.map(player => player.toString())
    }, [players]);

    const PLAYER_KEYWORD_DATA: KeywordDataMap = useMemo(() => {
        return Object.fromEntries(players.map((player) => {
            return [encodeString(player.toString()), [
                { style: "keyword-player-number", replacement: (player.index + 1).toString() },
                { replacement: " " },
                { style: "keyword-player", replacement: encodeString(player.name) }
            ]] as [string, TokenData[]]
        }))
    }, [players]);
    
    const PLAYER_SENDER_KEYWORD_DATA: KeywordDataMap = useMemo(() => {
        return Object.fromEntries(players.map((player) => {
            return [encodeString(player.toString()), [
                { style: "keyword-player-number", replacement: (player.index + 1).toString() },
                { replacement: " " },
                { style: "keyword-player-sender", replacement: encodeString(player.name) }
            ]] as [string, TokenData[]]
        }))
    }, [players]);

    const ROLE_LIST_KEYWORD_DATA: KeywordDataMap = useMemo(() => {
        return Object.fromEntries(props.roleList.map((outline, index) => {
            return [`${index + 1}: ` + translateRoleOutline(outline, playerNames), [
                { style: "keyword-outline-number", replacement: (index + 1).toString() },
                { replacement: " " },
                { style: "keyword-outline", replacement: getStyledHtmlFromString(translateRoleOutline(outline, playerNames), PLAYER_KEYWORD_DATA, {}) },
            ]] as [string, TokenData[]]
        }))
    }, [props.roleList]);

    return <div className="game-results">
        <div className="game-summary">
            <div className="game-summary-info">
                <div className="conclusion">
                    <StyledText>{translate("menu.gameResults.conclusion", translateConclusion(props.conclusion))}</StyledText>
                    <StyledText>{props.playerSynopsis.won ? translate("menu.gameResults.youWon") : translate("menu.gameResults.youLost")}</StyledText>
                </div>
            </div>
            <h2>
                <StyledText>
                    {translate("menu.gameResults.story")}
                </StyledText>
            </h2>
            <div className="story chat-menu-colors">
                <div className="beat graveyard-menu-colors">
                    <StyledText>
                        {translate("menu.gameResults.story.beat.initial",
                            translateRoleOutline(props.roleList[props.playerSynopsis.outlineAssignment.roleOutlineIndex], props.playerNames),
                            translate("role." + props.playerSynopsis.outlineAssignment.role + ".name"),
                            translateWinCondition(props.playerSynopsis.outlineAssignment.winCondition),
                            translateInsiderGroups(props.playerSynopsis.outlineAssignment.insiderGroups, true)
                        )}
                    </StyledText>
                </div>
                {props.playerSynopsis.crumbs.map((crumb, i) => <Crumb
                    index={i}
                    key={i}
                    crumb={crumb}
                    playerNames={props.playerNames}
                    roleList={props.roleList}
                />)}
                <div className="beat">
                    <StyledText>
                        {translate("menu.gameResults.story.beat.won." + props.playerSynopsis.won)}
                    </StyledText>
                </div>
                <div className="beat will-menu-colors alibi">
                    {translate("menu.gameResults.yourAlibi")}
                    <ChatElement 
                        message={{ chatGroup: "all", variant: {
                            type: "normal",
                            block: true,
                            messageSender: {
                                type: "player",
                                player: props.playerSynopsis.index
                            },
                            text: props.playerSynopsis.latestAlibi
                        }}}
                        playerNames={playerNames as string[]}
                        roleList={props.roleList}
                        playerKeywordData={PLAYER_KEYWORD_DATA}
                        playerSenderKeywordData={PLAYER_SENDER_KEYWORD_DATA}
                        roleListKeywordData={ROLE_LIST_KEYWORD_DATA}
                    />
                </div>
            </div>
        </div>
        <div className="personal-summary">
            <StreakTracker progress={streakProgress}/>
        </div>
    </div>
}

function StreakTracker(props: Readonly<{
    progress: StreakProgress
}>): ReactElement {
    const streakWeeks = useMemo<StreakWeek[]>(() => {
        const completedWeeksToShow = Math.min(4, props.progress.length);
        const firstCompletedWeek = props.progress.length - completedWeeksToShow + 1;
        const completedWeeks = Array.from({ length: completedWeeksToShow }, (_, index) => {
            const weekNumber = firstCompletedWeek + index;
            return {
                weekNumber,
                current: weekNumber === props.progress.length,
                next: false
            };
        });

        return [
            ...completedWeeks,
            {
                weekNumber: props.progress.length + 1,
                current: false,
                next: true
            }
        ];
    }, [props.progress]);

    return <section className="streak-tracker">
        <div className="streak-flame" aria-hidden="true">🔥</div>
        <div className="streak-count">
            <strong>{props.progress.length}</strong>
            <span>{translate("menu.gameResults.streak.weeks")}</span>
        </div>
        <div className="streak-status">
            {props.progress.justDied
                ? translate("menu.gameResults.streak.justDied")
                : (
                    props.progress.advanced
                        ? translate("menu.gameResults.streak.advanced")
                        : translate("menu.gameResults.streak.saved")
                )
            }
        </div>
        <ol className="streak-week-track" aria-label={translate("menu.gameResults.streak.weekLabel")}>
            {streakWeeks.map(week => <li
                key={week.weekNumber}
                className={[
                    "streak-week",
                    week.current ? "current" : "",
                    week.next ? "next" : "complete"
                ].join(" ")}
            >
                <span className="streak-week-marker" />
                <span className="streak-week-label">
                    {week.next
                        ? translate("menu.gameResults.streak.nextWeek", week.weekNumber)
                        : translate("menu.gameResults.streak.weekNumber", week.weekNumber)
                    }
                </span>
            </li>)}
        </ol>
    </section>
}

function getStreakProgress(streak: Streak): StreakProgress {
    const now = Date.now();
    const todayStart = getDayStart(now);
    const thisWeekStart = getWeekStart(now);
    const lastPlayedWeekStart = streak.lastPlayed === null ? null : getWeekStart(streak.lastPlayed);

    let length = streak.length;
    let advanced = false;
    let justDied = false;

    if (lastPlayedWeekStart === null) {
        length = 1;
        advanced = true;
    } else if (lastPlayedWeekStart === thisWeekStart) {
        length = Math.max(1, streak.length);
    } else if (lastPlayedWeekStart === thisWeekStart - WEEK_IN_MS) {
        length = Math.max(1, streak.length) + 1;
        advanced = true;
    } else {
        length = 1;
        justDied = true;
        advanced = true;
    }

    return {
        length,
        advanced,
        lastPlayed: now,
        justDied
    }
}

function getDayStart(time: number): number {
    const date = new Date(time);
    date.setHours(0, 0, 0, 0);
    return date.getTime();
}

function getWeekStart(time: number): number {
    const date = new Date(getDayStart(time));
    date.setDate(date.getDate() - date.getDay());
    return date.getTime();
}

function Crumb(props: Readonly<{
    index: number,
    playerNames: string[],
    roleList: RoleList,
    crumb: SynopsisCrumb
}>): ReactElement {
    const crumb = props.crumb;

    const [className, inner]: [string, ReactNode] = useMemo(() => {
        if ("roleChange" in crumb) {
            return [
                'role-specific-colors',
                <StyledText>
                    {translate("menu.gameResults.story.beat.roleChange", translate("role." + crumb.roleChange + ".name"))}
                </StyledText>
            ]
        } else if ("winConditionChange" in crumb) {
            return [
                'will-menu-colors',
                <StyledText>
                    {translate("menu.gameResults.story.beat.winConditionChange", translateWinCondition(crumb.winConditionChange))}
                </StyledText>
            ] 
        } else if ("insiderGroupChange" in crumb) {
            return [
                'chat-menu-colors',
                <StyledText>
                    {translate("menu.gameResults.story.beat.insiderGroupChange", translateInsiderGroups(crumb.insiderGroupChange, true))}
                </StyledText>
            ]
        } else if ("died" in crumb) {
            return [
                'graveyard-menu-colors',
                <StyledText>
                    {translate("menu.gameResults.story.beat.died")}
                </StyledText>
            ]
        } else if ("grave" in crumb) {
            return [
                'graveyard-menu-colors',
                <>
                    {translate("menu.gameResults.story.beat.grave")}
                    <GraveComponent grave={crumb.grave} playerNames={props.playerNames} roleList={props.roleList}/>
                </>
            ]
        } else {
            return ['', "ERROR"]
        }
    }, [])

    return <div className={"beat " + className}>
        {inner}
    </div>
}
