import React, { ReactElement, useCallback, useEffect, useMemo, useRef, useState } from "react";
import translate from "../game/lang";
import "./wiki.css";
import { Role, getMainRoleSetFromRole } from "../game/roleState.d";
import GAME_MANAGER, { regEscape } from "..";
import WikiArticle, { getSearchStrings, PageCollection } from "./WikiArticle";
import { ARTICLES, WikiArticleLink, getArticleTitle, wikiPageIsEnabled } from "./WikiArticleLink";
import StyledText from "./StyledText";
import Icon from "./Icon";
import { ContentMenu, MenuController } from "../menu/game/GameScreen";
import { AnchorController } from "../menu/Anchor";
import WikiCoverCard from "./WikiCoverCard";
import { getAllRoles } from "../game/roleListState.d";
import { useLobbyOrGameState } from "./useHooks";
import { MODIFIERS, ModifierType } from "../game/gameState.d";
import Masonry from "react-responsive-masonry";
import CheckBox from "./CheckBox";


export function setWikiSearchPage(page: WikiArticleLink, anchorController: AnchorController, menuController?: MenuController) {
    if (GAME_MANAGER.wikiArticleCallbacks.length === 0) {
        if (menuController?.canOpen(ContentMenu.WikiMenu)) {
            menuController.openMenu(ContentMenu.WikiMenu, () => {
                GAME_MANAGER.setWikiArticle(page);
            });
        } else {
            anchorController.setCoverCard(<WikiCoverCard initialWikiPage={page}/>)
        }
    } else {
        GAME_MANAGER.setWikiArticle(page);
    }
}


export default function Wiki(props: Readonly<{
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],
    initialWikiPage?: WikiArticleLink,
    onPageChange?: (page: WikiArticleLink | null) => void,
    static?: boolean
}>): ReactElement {

    const [searchQuery, setSearchQuery] = useState("");
    const [article, setArticle] = useState<WikiArticleLink | null>(props.initialWikiPage ?? null);
    const [history, setHistory] = useState<WikiArticleLink[]>([]);

    useEffect(() => {
        props.onPageChange && props.onPageChange(article);
    }, [article, props])

    const chooseArticle = useCallback((page: WikiArticleLink | null) => {
        if (page !== null) {
            if(history[history.length - 1] !== page)
                setHistory([...history, page]);
            if(history.length > 50)
                setHistory(history.slice(1));
            
        }
        setArticle(page);
    }, [history]);

    useEffect(() => {
        GAME_MANAGER.addSetWikiArticleCallback(chooseArticle);

        return () => GAME_MANAGER.removeSetWikiArticleCallback(chooseArticle);
    }, [setArticle, chooseArticle]);

    // This makes sure you can call GAME_MANAGER.setWikiArticle immediately after this component is added
    GAME_MANAGER.addSetWikiArticleCallback(chooseArticle);
    setTimeout(() => {
        GAME_MANAGER.removeSetWikiArticleCallback(chooseArticle);
    }, 100)

    function goBack() {
        if (history.length > 1) {
            let newHistory = [...history];
            newHistory.pop();
            setHistory(newHistory);
            setArticle(newHistory[newHistory.length - 1]);
        }else{
            setHistory([]);
            setArticle(null);
        }
    }

    return <div className="wiki-search">
        <WikiSearchBar 
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            onBack={goBack}
            onClear={() => {setSearchQuery(""); setArticle(null); setHistory([]);}}
        />
        {
            article === null ?
            <WikiSearchResults 
                searchQuery={searchQuery}
                enabledRoles={props.enabledRoles}
                enabledModifiers={props.enabledModifiers}
                onChooseArticle={chooseArticle}
                static={props.static === true}
            />
            :
            <WikiArticle article={article}/>
        }
    </div>
}

function WikiSearchBar(props: Readonly<{
    searchQuery: string,
    onSearchChange: (search: string) => void,
    onBack: () => void,
    onClear: () => void,
}>): ReactElement {
    return <div className="wiki-search-bar">
        <button tabIndex={-1} onClick={() => props.onBack()}>
            <Icon>arrow_back</Icon>
        </button>
        <input type="text" value={props.searchQuery}
            onChange={(e)=>{
                props.onSearchChange(e.target.value.trimStart())}
            }
            placeholder={translate("menu.wiki.search.placeholder")}
        />
        <button tabIndex={-1} onClick={props.onClear}>
            <Icon>close</Icon>
        </button>
    </div>
}

function WikiSearchResults(props: Readonly<{
    searchQuery: string,
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],
    onChooseArticle: (article: WikiArticleLink) => void,
    static: boolean
}>): ReactElement {
    const enabledRoles = useLobbyOrGameState(
        gameState => gameState.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;
    const enabledModifiers = useLobbyOrGameState(
        gameState => gameState.enabledModifiers,
        ["enabledModifiers"],
        MODIFIERS as any as ModifierType[]
    )!;

    const [hideDisabled, setHideDisabled] = useState(true);

    const getSearchResults = useCallback((search: string) => {
        const out = [
            ...ARTICLES.filter((page) => {return RegExp(regEscape(search.trim()), 'i').test(getArticleTitle(page))}), 
            ...ARTICLES.filter((page) => {return getSearchStrings(page).some((str) => RegExp(regEscape(search.trim()), 'i').test(str))})
        ];
        return out.filter((item, index) => out.indexOf(item) === index);
    }, []);

    const results = useMemo(() => 
        getSearchResults(props.searchQuery),
    [props.searchQuery, getSearchResults])

    return <div className="wiki-results" tabIndex={-1}>
        {!props.static && <label className="centered-label">
            {translate("hideDisabled")}
            <CheckBox 
                checked={hideDisabled} 
                onChange={checked => setHideDisabled(checked)}
            />
        </label>}
        {props.searchQuery === ""
            ? <WikiMainPage 
                enabledRoles={enabledRoles} 
                enabledModifiers={enabledModifiers} 
                hideDisabled={hideDisabled}
                articles={results} 
                onChooseArticle={props.onChooseArticle}
            /> : <div>
                {results
                    .filter(page => wikiPageIsEnabled(page, enabledRoles, enabledModifiers) || !hideDisabled)
                    .map(page => <WikiSearchResult key={page} 
                        page={page} 
                        className={wikiPageIsEnabled(page, enabledRoles, enabledModifiers) ? "" : "keyword-disabled"} 
                        onChooseArticle={() => props.onChooseArticle(page)}
                    />)}
            </div>}
    </div>
}

function WikiMainPage(props: Readonly<{
    articles: WikiArticleLink[],
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],
    hideDisabled: boolean,
    onChooseArticle: (article: WikiArticleLink) => void
}>): ReactElement {
    const articlePartitions = useMemo(() => 
        partitionWikiPages(props.articles, props.enabledRoles, props.enabledModifiers),
    [props.articles, props.enabledRoles, props.enabledModifiers]);

    const ref = useRef<HTMLDivElement>(null);

    const [columnCount, setColumnCount] = useState(1);

    useEffect(() => {
        const redetermineColumnWidths = () => {
            if (ref.current) {
                setColumnCount(Math.max(Math.floor(ref.current.clientWidth / 300), 1))
            }
        }

        const resizeObserver = new ResizeObserver(redetermineColumnWidths)

        redetermineColumnWidths()

        setTimeout(() => {
            resizeObserver.observe(ref.current!);
        })
        return resizeObserver.unobserve(ref.current!)
    }, [ref])

    return <div ref={ref} className="wiki-main-page">
        <Masonry columnsCount={columnCount}>
            {Object.entries(articlePartitions)
                .filter(([category]) => category !== "uncategorized")
                .map(([category, pages]) => {
                    return <div className="masonry-item" key={category}>
                        <PageCollection 
                            title={translate(`wiki.category.${category}`)}
                            pages={pages
                                .filter(page => wikiPageIsEnabled(page, props.enabledRoles, props.enabledModifiers) || !props.hideDisabled)}
                            enabledRoles={props.enabledRoles}
                            enabledModifiers={props.enabledModifiers}
                        />
                    </div>
                })}
        </Masonry>
        <PageCollection 
            title={translate(`wiki.category.uncategorized`)}
            pages={articlePartitions["uncategorized"]
                .filter(page => wikiPageIsEnabled(page, props.enabledRoles, props.enabledModifiers) || !props.hideDisabled)}
            enabledRoles={props.enabledRoles}
            enabledModifiers={props.enabledModifiers}
        />
    </div>
}

export const WIKI_CATEGORIES = [
    "categories", "town", "mafia", "cult", "neutral", "minions", "fiends", "modifiers", "abilities", "strategies", "menus", "phases", "trial"
] as const;
export type WikiCategory = (typeof WIKI_CATEGORIES)[number]

type WikiPagePartitions = Record<WikiCategory | "uncategorized", WikiArticleLink[]>;

export function partitionWikiPages(
    wikiPages: WikiArticleLink[],
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],
    sort?: boolean
): WikiPagePartitions {
    const partitions: WikiPagePartitions = Object.fromEntries([
        ...WIKI_CATEGORIES.map(a => [a, []]),
        ["uncategorized", []]
    ]) as WikiPagePartitions;

    for (const wikiPage of wikiPages) {
        const articleType = wikiPage.split("/")[0];

        let categories: (WikiCategory | "uncategorized")[] = [];

        if (articleType === "role") {
            const role = wikiPage.split("/")[1] as Role;
            categories.push(getCategoryForRole(role));

        } else if (articleType === "modifier") {
            categories.push("modifiers")
        } else if (articleType === "category") {
            categories.push("categories")
        }

        if (wikiPage === "standard/mafia") {
            categories.push("mafia")
        } else if (wikiPage === "standard/cult") {
            categories.push("cult")
        }
        
        if ([
            "standard/backup", "standard/block", "standard/convert", "standard/douse", "standard/forged",
            "standard/frame", "standard/haunt", "standard/hypnotize", "standard/interview", "standard/detain",
            "standard/marionette", "standard/obscured", "standard/possess", 
            "standard/guard", "standard/fragileVest", "standard/rampage", "standard/report", "standard/roleblock", "standard/silenced",
            "standard/spiral", "standard/syndicateGunItem", "standard/transport", "standard/ward",
            "standard/forfeitNominationVote", "standard/aura", "standard/fastForward", "standard/appearedVisit", 
            "standard/defense", "standard/confused", "standard/trial",
        ].includes(wikiPage)) {
            categories.push("abilities")
        }
        
        if ([
            "standard/claim", "standard/claimswap", "standard/vfr", "standard/passcode",
        ].includes(wikiPage)) {
            categories.push("strategies")
        }

        if ([
            "standard/playerList", "standard/gameMode", "standard/outlineList", "standard/alibi", "standard/chat", "standard/controller"
        ].includes(wikiPage)) {
            categories.push("menus")
        }

        if ([
            "category/trial", "standard/briefing", "standard/night", "standard/obituary",
            "standard/discussion", "standard/nomination", "standard/adjournment", "standard/testimony",
            "standard/judgement", "standard/dusk", "standard/finalWords"
        ].includes(wikiPage)) {
            categories.push("phases")
        }

        if ([
            "standard/nomination", "standard/adjournment", "standard/testimony", "standard/judgement", "standard/finalWords",
        ].includes(wikiPage)) {
            categories.push("trial")
        }

        if (categories.length === 0) {
            categories.push("uncategorized")
        }

        for (const category of categories) {
            partitions[category].push(wikiPage)
        }
    }

    if (sort !== false) {
        const sortFunction = getWikiPageSortFunction(enabledRoles, enabledModifiers);
    
        for (const category of Object.keys(partitions) as WikiCategory[]) {
            partitions[category].sort(sortFunction);
        }
    }

    return partitions;
}

function getCategoryForRole(role: Role): WikiCategory {
    return getMainRoleSetFromRole(role) as WikiCategory;
}

function getWikiPageSortFunction(
    enabledRole: Role[],
    enabledModifiers: ModifierType[]
): (first: WikiArticleLink, second: WikiArticleLink) => number {
    return (first, second) => wikiPageSortFunction(first, second, enabledRole, enabledModifiers)
}

function wikiPageSortFunction(
    first: WikiArticleLink,
    second: WikiArticleLink,
    enabledRoles: Role[],
    enabledModifiers: ModifierType[]
): number {
    const isPageEnabled = (page: WikiArticleLink) => wikiPageIsEnabled(page, enabledRoles, enabledModifiers);

    if (isPageEnabled(first) && !isPageEnabled(second)) {
        return -1;
    } else if (!isPageEnabled(first) && isPageEnabled(second)) {
        return 1
    } else {
        return getArticleTitle(first).localeCompare(getArticleTitle(second))
    }
}

function WikiSearchResult(props: Readonly<{
    page: WikiArticleLink,
    className?: string,
    onChooseArticle: (article: WikiArticleLink) => void
}>): ReactElement {
    return <button key={props.page} onClick={() => props.onChooseArticle(props.page)}>
        <StyledText noLinks={true} className={props.className}>
            {getArticleTitle(props.page)}
        </StyledText>
</button>
}