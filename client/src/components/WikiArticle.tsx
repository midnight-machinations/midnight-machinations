import { ReactElement, ReactNode, useEffect, useRef, useState } from "react";
import { Role, roleJsonData } from "../game/roleState.d";
import React from "react";
import translate, { langText, translateChecked } from "../game/lang";
import StyledText, { DUMMY_NAMES_KEYWORD_DATA, DUMMY_NAMES_SENDER_KEYWORD_DATA, DUMMY_ROLE_LIST_KEYWORD_DATA, StyledTextProps } from "./StyledText";
import { ROLE_SETS, RoleList, getAllRoles, getRolesFromRoleSet } from "../game/roleListState.d";
import ChatElement from "./ChatMessage";
import DUMMY_NAMES from "../resources/dummyNames.json";
import { ARTICLES, GeneratedArticle, getArticleTitle, WikiArticleLink, wikiPageIsEnabled } from "./WikiArticleLink";
import "./wiki.css";
import GAME_MANAGER, { replaceMentions } from "..";
import { useLobbyOrGameState } from "./useHooks";
import DetailsSummary from "./DetailsSummary";
import { partitionWikiPages, WikiCategory } from "./Wiki";
import { MODIFIERS, ModifierID } from "../game/modifiers";
import DUMMY_ROLE_LIST from "../resources/dummyRoleList.json";
import Masonry from "react-responsive-masonry";

function WikiStyledText(props: Omit<StyledTextProps, 'markdown' | 'playerKeywordData'>): ReactElement {
    return <StyledText {...props} markdown={true} playerKeywordData={DUMMY_NAMES_KEYWORD_DATA} roleListKeywordData={DUMMY_ROLE_LIST_KEYWORD_DATA}/>
}

export default function WikiArticle(props: {
    article: WikiArticleLink
}): ReactElement {
    
    const path = props.article.split('/');

    switch (path[0]) {
        case "role": {
            const role = path[1] as Role;
            const roleData = roleJsonData()[role];
            const chatMessages = roleData.chatMessages;
            const exampleAlibi = translateChecked("wiki.article.role."+role+".exampleAlibi");
            const exampleAlibiDescription = translateChecked("wiki.article.role."+role+".exampleAlibi.description");

            return <section className="wiki-article">
                <div>
                    <WikiStyledText>
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
                        <WikiStyledText>
                            {"### "+translate("wiki.article.standard.aura.title")+": "+translate(roleData.aura+"Aura")+"\n"}
                        </WikiStyledText>
                    }
                    {roleData.armor && 
                        <WikiStyledText>
                            {"### "+translate("defense")+": "+translate("defense.armored")+"\n"}
                        </WikiStyledText>
                    }
                    {roleData.maxCount !== null &&
                        <WikiStyledText>
                        {"### "+translate("wiki.article.standard.roleLimit.title")+": "+(roleData.maxCount)+"\n"}
                        </WikiStyledText>
                    }
                </div>
                {chatMessages.length!==0 && <div className="wiki-message-section">
                    <WikiStyledText>
                        {"### "+translate("wiki.article.role.chatMessages")+"\n"}
                    </WikiStyledText>
                    {chatMessages.map((msgvariant, i)=>
                        <ChatElement key={i} 
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
                        />
                    )}
                </div>}
                {exampleAlibi && <div className="wiki-message-section">
                    <WikiStyledText>
                        {"### "+translate("wiki.article.role.exampleAlibi")+"\n"}
                    </WikiStyledText>
                    {exampleAlibiDescription && <WikiStyledText>
                        {replaceMentions(exampleAlibiDescription, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}
                    </WikiStyledText>}
                    <blockquote>
                        <WikiStyledText>{replaceMentions(exampleAlibi, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}</WikiStyledText>
                    </blockquote>
                </div>}
                <DetailsSummary 
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
                </DetailsSummary>
            </section>
        }
        case "category": 
            return <CategoryArticle category={path[1] as WikiCategory}/>
        case "standard":
        case "modifier": {
            const articleType = path[0];
            return <section className="wiki-article">
                <WikiStyledText className="wiki-article-standard">
                    {"# "+translate(`wiki.article.${articleType}.${props.article.split("/")[1]}.title`)+"\n"}
                    {replaceMentions(translate(`wiki.article.${articleType}.${props.article.split("/")[1]}.text`), DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string}
                </WikiStyledText>
            </section>
        }
        case "generated":
            return <section className="wiki-article">
                <GeneratedArticleElement article={path[1] as GeneratedArticle}/>
            </section>
    }

    return <></>;
}

function CategoryArticle(props: Readonly<{ category: WikiCategory }>): ReactElement {
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

    return <section className="wiki-article">
        <WikiStyledText className="wiki-article-standard">
            {"# "+title+"\n"}
            {description ? replaceMentions(description, DUMMY_NAMES, (DUMMY_ROLE_LIST as RoleList)) as string : ""}
        </WikiStyledText>
        <PageCollection 
            title={title}
            pages={partitionWikiPages(ARTICLES, enabledRoles, enabledModifiers)[props.category] ?? []}
            enabledRoles={enabledRoles}
            enabledModifiers={enabledModifiers}
        />
    </section>
}

export function PageCollection(props: Readonly<{
    title: string,
    pages: WikiArticleLink[],
    enabledRoles: Role[],
    enabledModifiers: ModifierID[],
    children?: ReactNode
}>): ReactElement | null {
    if (props.pages.length === 0) {
        return null;
    }
    
    return <>
        <h3 className="wiki-search-divider">
            <StyledText>{props.title}</StyledText>
        </h3>
        {props.children}
        {props.pages.map((page) => {
            return <button key={page} className={wikiPageIsEnabled(page, props.enabledRoles, props.enabledModifiers) ? "" : "keyword-disabled"} 
                onClick={() => GAME_MANAGER.setWikiArticle(page)}
            >
                <StyledText noLinks={true}>{getArticleTitle(page)}</StyledText>
            </button>
        })}
    </>
}


function GeneratedArticleElement(props: Readonly<{ article: GeneratedArticle }>): ReactElement {
    switch(props.article){
        case "roleSet":
            return <RoleSetArticle />
        case "all_text":
            return <pre>
                <h1>{translate("wiki.article.generated.all_text.title")}</h1>
                <StyledText className="code">{langText.substring(1, langText.length - 1)}</StyledText>
            </pre>;
    }
}

function RoleSetArticle(): ReactElement {
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
            <WikiStyledText>{"# "+translate("wiki.article.generated.roleSet.title")}</WikiStyledText>
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
                    >
                        {description && <p><StyledText>{description}</StyledText></p>}
                    </PageCollection>
                </div>
            })}
        </Masonry>
        <WikiStyledText key={"extra"}>
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
            return [
                translate(`wiki.article.${path[0]}.${path[1]}.title`),
                translate(`wiki.article.${path[0]}.${path[1]}.text`),
            ]
        }
        case "generated":
            return getSearchStringsGenerated(path[1] as GeneratedArticle);
        default: // Categories don't show up in search results
            return [];
    }
}