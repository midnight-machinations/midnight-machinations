import { ReactElement, ReactNode, useContext, useEffect, useRef, useState } from "react";
import { Role, roleJsonData } from "../game/roleState.d";
import React from "react";
import translate, { langText, translateChecked, translateCustomWikiArticle } from "../game/lang";
import StyledText, { DUMMY_NAMES_KEYWORD_DATA, DUMMY_NAMES_SENDER_KEYWORD_DATA, DUMMY_ROLE_LIST_KEYWORD_DATA, StyledTextProps } from "./StyledText";
import { ROLE_SETS, RoleList, getAllRoles, getRolesFromRoleSet } from "../game/roleListState.d";
import ChatElement from "./ChatMessage";
import DUMMY_NAMES from "../resources/dummyNames.json";
import { getAllWikiArticles, GeneratedArticle, getArticleTitle, WikiArticleLink, wikiPageIsEnabled } from "./WikiArticleLink";
import "./wiki.css";
import GAME_MANAGER, { replaceMentions } from "..";
import { useLobbyOrGameState } from "./useHooks";
import DetailsSummary from "./DetailsSummary";
import { partitionWikiPages, WikiCategory, WikiDisabledFilter } from "./Wiki";
import { MODIFIERS, ModifierID } from "../game/modifiers";
import DUMMY_ROLE_LIST from "../resources/dummyRoleList.json";
import Masonry from "react-responsive-masonry";
import Popover from "./Popover";
import { dropdownPlacementFunction } from "./Select";
import WikiArticleTooltip, { getArticleTooltip } from "./WikiArticleTooltip";
import { CtrlPressedContext } from "../menu/Anchor";

function WikiStyledText(props: Omit<StyledTextProps, 'markdown' | 'playerKeywordData'>): ReactElement {
    return <StyledText {...props} markdown={true} playerKeywordData={DUMMY_NAMES_KEYWORD_DATA} roleListKeywordData={DUMMY_ROLE_LIST_KEYWORD_DATA}/>
}

export default function WikiArticle(props: {
    article: WikiArticleLink
    className?: string
    noLinks?: boolean
}): ReactElement {
    
    const path = props.article.split('/');

    switch (path[0]) {
        case "role": {
            const role = path[1] as Role;
            const roleData = roleJsonData()[role];
            const chatMessages = roleData.chatMessages;
            const exampleAlibi = translateChecked("wiki.article.role."+role+".exampleAlibi");
            const exampleAlibiDescription = translateChecked("wiki.article.role."+role+".exampleAlibi.description");

            return <section className={"wiki-article " + (props.className ?? "")}>
                <div>
                    <WikiStyledText noLinks={props.noLinks}>
                        {"# "+translate("role."+role+".name")+"\n"}
                        {"### "+roleData.roleSets.map((roleSet)=>{return translate(roleSet)}).join(" | ")+"\n"}

                        {"### "+translate("wiki.article.role.reminder")+"\n"}
                        {replaceMentions(translateChecked("wiki.article.role."+role+".reminder") ?? translate("wiki.article.role.noReminder"), DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList))+"\n"}

                        {translateChecked("wiki.article.role."+role+".lore")!==null?("### "+translate("wiki.article.role.lore")+"\n"):""}
                        {replaceMentions(translateChecked("wiki.article.role."+role+".lore") ?? "", DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList))+"\n"}

                        {"### "+translate("wiki.article.role.guide")+"\n"}
                        {replaceMentions(translateChecked("wiki.article.role."+role+".guide") ?? translate("wiki.article.role.noGuide"), DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList))+"\n"}
                    </WikiStyledText>
                </div>
                <div>
                    {roleData.aura &&
                        <WikiStyledText noLinks={props.noLinks}>
                            {"### "+translate("wiki.article.standard.aura.title")+": "+translate(roleData.aura+"Aura")+"\n"}
                        </WikiStyledText>
                    }
                    {roleData.armor && 
                        <WikiStyledText noLinks={props.noLinks}>
                            {"### "+translate("defense")+": "+translate("defense.armored")+"\n"}
                        </WikiStyledText>
                    }
                    {roleData.maxCount !== null &&
                        <WikiStyledText noLinks={props.noLinks}>
                        {"### "+translate("wiki.article.standard.roleLimit.title")+": "+(roleData.maxCount)+"\n"}
                        </WikiStyledText>
                    }
                </div>
                {chatMessages.length!==0 && <div className="wiki-message-section">
                    <WikiStyledText noLinks={props.noLinks}>
                        {"### "+translate("wiki.article.role.chatMessages")+"\n"}
                    </WikiStyledText>
                    {chatMessages.map((msgvariant, i)=>
                        <ChatElement key={i}
                            canCopyPaste={true}
                            message={
                                {
                                    variant: msgvariant,
                                    chatGroup: "all",
                                }
                            } 
                            playerNames={DUMMY_NAMES} 
                            playerKeywordData={DUMMY_NAMES_KEYWORD_DATA}
                            playerSenderKeywordData={DUMMY_NAMES_SENDER_KEYWORD_DATA}
                            roleList={DUMMY_ROLE_LIST as RoleList}
                            roleListKeywordData={DUMMY_ROLE_LIST_KEYWORD_DATA}
                            noLinks={props.noLinks}
                        />
                    )}
                </div>}
                {exampleAlibi && <div className="wiki-message-section">
                    <WikiStyledText noLinks={props.noLinks}>
                        {"### "+translate("wiki.article.role.exampleAlibi")+"\n"}
                    </WikiStyledText>
                    {exampleAlibiDescription && <WikiStyledText noLinks={props.noLinks}>
                        {replaceMentions(exampleAlibiDescription, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}
                    </WikiStyledText>}
                    <blockquote>
                        <WikiStyledText noLinks={props.noLinks}>
                            {replaceMentions(exampleAlibi, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}
                        </WikiStyledText>
                    </blockquote>
                </div>}
                {!props.noLinks && <DetailsSummary 
                    summary={translate("wiki.article.role.details")}
                >
                    <WikiStyledText>
                        {"### "+translate("wiki.article.role.abilities")+"\n"}
                        {(translateChecked("wiki.article.role."+role+".abilities") ?? translate("wiki.article.role.noAbilities"))+"\n"}

                        {"### "+translate("wiki.article.role.attributes")+"\n"}
                        {(translateChecked("wiki.article.role."+role+".attributes") ?? translate("wiki.article.role.noAttributes"))+"\n"}

                        {"### "+translate("wiki.article.role.extra")+"\n"}
                        {(translateChecked("wiki.article.role."+role+".extra") ?? translate("wiki.article.role.noExtra"))+"\n"}

                        {"### "+translate("wiki.article.standard.roleLimit.title")+": "+(roleData.maxCount === null ? translate("none") : roleData.maxCount)+"\n"}
                        {"### "+translate("defense")+": "+translate("defense."+(roleData.armor ? "armored" : "none"))+"\n"}
                        {"### "+translate("wiki.article.standard.aura.title")+": "+(roleData.aura?translate(roleData.aura+"Aura"):translate("none"))+"\n"}
                    </WikiStyledText>
                </DetailsSummary>}
            </section>
        }
        case "category": 
            return <CategoryArticle noLinks={props.noLinks} category={path[1] as WikiCategory} className={props.className}/>
        case "standard":
        case "modifier": {
            const title = translateChecked(`wiki.article.${path[0]}.${props.article.split("/")[1]}.title`);
            const text = translateChecked(`wiki.article.${path[0]}.${props.article.split("/")[1]}.text`);

            let body = "ERROR: Could not find article.";

            if (!title && !text) {
                body = translateCustomWikiArticle(props.article);
            } else if (title && text) {
                body = `# ${title}\n${replaceMentions(text, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}`;
            }
            return <section className={"wiki-article " + (props.className ?? "")}>
                <WikiStyledText className="wiki-article-standard" noLinks={props.noLinks}>
                    {body}
                </WikiStyledText>
            </section>
        }
        case "generated":
            return <section className={"wiki-article " + (props.className ?? "")}>
                <GeneratedArticleElement noLinks={props.noLinks} article={path[1] as GeneratedArticle}/>
            </section>
    }

    return <></>;
}

function CategoryArticle(props: Readonly<{ noLinks?: boolean, category: WikiCategory, className?: string }>): ReactElement {
    const title = translate(`wiki.category.${props.category}`);
    const description = translateChecked(`wiki.category.${props.category}.text`);

    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;

    const enabledModifiers = useLobbyOrGameState(
        state => state.modifierSettings.keys(),
        ["modifierSettings"],
        MODIFIERS as any as ModifierID[]
    )!;

    return <section className={"wiki-article " + (props.className ?? "")}>
        <WikiStyledText noLinks={props.noLinks} className="wiki-article-standard">
            {"# "+title+"\n"}
            {description ? replaceMentions(description, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string : ""}
        </WikiStyledText>
        <PageCollection 
            title={title}
            pages={partitionWikiPages(getAllWikiArticles(), enabledRoles, enabledModifiers)[props.category] ?? []}
            enabledRoles={enabledRoles}
            enabledModifiers={enabledModifiers}
            noLinks={props.noLinks}
        />
    </section>
}

export function PageCollection(props: Readonly<{
    title: string,
    pages: WikiArticleLink[],
    enabledRoles: Role[],
    enabledModifiers: ModifierID[],
    children?: ReactNode,
    wikiDisabledFilter?: [string, WikiDisabledFilter] | "default",
    noLinks?: boolean
}>): ReactElement | null {
    if (props.pages.length === 0) {
        return null;
    }
    
    return <>
        <h3 className="wiki-search-divider">
            <StyledText noLinks={props.noLinks}>{props.title}</StyledText>
        </h3>
        {props.children}
        {!props.noLinks && props.pages.map((page) => {
            return <PageButton
                key={page}
                page={page}
                enabledRoles={props.enabledRoles}
                enabledModifiers={props.enabledModifiers}
                wikiDisabledFilter={props.wikiDisabledFilter}
            />
        })}
        {props.noLinks && props.pages.map((page) => {
            return <div key={page} className={"placard " + (wikiPageIsEnabled(page, props.enabledRoles, props.enabledModifiers, props.wikiDisabledFilter) ? "" : "keyword-disabled")}>
                <StyledText noLinks={true}>{getArticleTitle(page)}</StyledText>
            </div>
        })}
    </>
}

function PageButton(props: Readonly<{
    page: WikiArticleLink,
    enabledRoles: Role[],
    enabledModifiers: ModifierID[],
    wikiDisabledFilter?: [string, WikiDisabledFilter] | "default"
}>): ReactElement {
    const isCtrlPressed = useContext(CtrlPressedContext) ?? false;

    const articleTooltip = React.useMemo(() => {
        if (isCtrlPressed === true) {
            return <WikiArticle noLinks={true} article={props.page} className="wiki-article-tooltip" />;
        } else {
            const tooltip = getArticleTooltip(props.page);
            if (tooltip === null) {
                return null;
            }
            return <WikiArticleTooltip tooltip={tooltip} />;
        }
    }, [isCtrlPressed, props.page]);   

    const buttonRef = useRef<HTMLButtonElement>(null);

    const [hovering, setHovering] = React.useState<boolean>(false);

    const handleFocus = (event: any) => {
        setHovering(true);
    };

    const handleUnfocus = (event: any) => {
        setHovering(false);
    };

    return <>
        <button ref={buttonRef} key={props.page} className={wikiPageIsEnabled(props.page, props.enabledRoles, props.enabledModifiers, props.wikiDisabledFilter) ? "" : "keyword-disabled"} 
            onClick={() => GAME_MANAGER.setWikiArticle(props.page)}
            onMouseEnter={handleFocus}
            onMouseLeave={handleUnfocus}
            onFocus={handleFocus}
            onBlur={handleUnfocus}
        >
            <StyledText noLinks={true}>{getArticleTitle(props.page)}</StyledText>
        </button>
        {articleTooltip !== null && <Popover
            open={hovering && articleTooltip !== null}
            setOpenOrClosed={setHovering}
            anchorForPositionRef={buttonRef}
            onRender={(popover, anchor) => dropdownPlacementFunction(popover, anchor, null)}
            className="wiki-article-tooltip-popover"
        >
            {articleTooltip}
        </Popover>}
    </>
}


function GeneratedArticleElement(props: Readonly<{ noLinks?: boolean, article: GeneratedArticle }>): ReactElement {
    switch(props.article){
        case "roleSet":
            return <RoleSetArticle noLinks={props.noLinks} />
        case "all_text":
            return <pre>
                <h1>{translate("wiki.article.generated.all_text.title")}</h1>
                <StyledText noLinks={props.noLinks} className="code">{langText.substring(1, langText.length - 1)}</StyledText>
            </pre>;
    }
}

function RoleSetArticle(props: Readonly<{ noLinks?: boolean }>): ReactElement {
    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"],
        getAllRoles()
    )!;

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

    return <div ref={ref} className="role-set-article">
        <section key="title">
            <WikiStyledText noLinks={props.noLinks} >{"# "+translate("wiki.article.generated.roleSet.title")}</WikiStyledText>
        </section>
        <Masonry columnsCount={columnCount}>
            {ROLE_SETS.filter(set=>set!=="any").map(set => {
                const description = translateChecked(`${set}.description`);
                return <div key={set} className="masonry-item">
                    <PageCollection
                        title={translate(set)}
                        pages={getRolesFromRoleSet(set).map(role => `role/${role}` as WikiArticleLink)}
                        enabledRoles={enabledRoles}
                        enabledModifiers={[]}
                        noLinks={props.noLinks}
                    >
                        {description && <p><StyledText noLinks={props.noLinks}>{description}</StyledText></p>}
                    </PageCollection>
                </div>
            })}
        </Masonry>
        <WikiStyledText noLinks={props.noLinks} key={"extra"}>
            {translate("wiki.article.generated.roleSet.extra", Object.keys(roleJsonData()).length)}
        </WikiStyledText>
    </div>;
}

function getSearchStringsGenerated(article: GeneratedArticle): string[]{
    switch(article){
        case "roleSet": {
            let out = [translate("wiki.article.generated.roleSet.title")];
            for(let set of ROLE_SETS){
                out.push(translate(set));
            }
            return out;
        }
        case "all_text":
            return [];
    }
}

export function getSearchStrings(article: WikiArticleLink): string[]{
    const path = article.split('/');

    switch (path[0]) {
        case "role": {
            const role = path[1] as Role;
            const roleData = roleJsonData()[role];
            let out = [];

            out.push(translate("role."+role+".name"));

            for(let roleSet of roleData.roleSets){
                out.push(translate(roleSet));
            }

            let guide = translateChecked("wiki.article.role."+role+".guide");
            if(guide)
                out.push(guide);
            if(roleData.armor){
                out.push(translate("defense.armored"));
                out.push(translate("defense"));
            }
            let abilities = translateChecked("wiki.article.role."+role+".abilities");
            if(abilities)
                out.push(abilities);
            let attributes = translateChecked("wiki.article.role."+role+".attributes");
            if(attributes)
                out.push(attributes);
            let extra = translateChecked("wiki.article.role."+role+".extra");
            if(extra)
                out.push(extra);
            let roleLimit = roleData.maxCount !== null;
            if(roleLimit)
                out.push(translate("wiki.article.standard.roleLimit.title"));

            return out;            
        }
        case "modifiers":
        case "standard": {
            const title = translateChecked(`wiki.article.${path[0]}.${path[1]}.title`);

            if (title === null) {
                // This is a custom article
                return [getArticleTitle(article)];
            } else {
                // This is defined in en_us.json
                return [
                    title,
                    translate(`wiki.article.${path[0]}.${path[1]}.text`),
                ]
            }
        }
        case "generated":
            return getSearchStringsGenerated(path[1] as GeneratedArticle);
        default: // Categories don't show up in search results
            return [];
    }
}
