import { MODIFIERS, ModifierID } from "../game/modifiers";
import translate, { customWikiArticles, langJson, translateChecked, translateCustomWikiArticle } from "../game/lang";
import { Role, roleJsonData } from "../game/roleState.d";
import { partitionWikiPages, WIKI_CATEGORIES, WikiCategory, WikiDisabledFilter } from "./Wiki";
import "./wiki.css";
import { getAllRoles } from "../game/roleListState.d"

export type WikiArticleLink = 
    `role/${Role}` | 
    `modifier/${ModifierID}` |
    `category/${WikiCategory}` |
    `standard/${string}` |
    `generated/${GeneratedArticle}`;

const GENERATED_ARTICLES = ["roleSet", "all_text"] as const;
export type GeneratedArticle = typeof GENERATED_ARTICLES[number];

const CONSTANT_ARTICLES: WikiArticleLink[] = 
    WIKI_CATEGORIES.map(category => `category/${category}`)
        .concat(getAllRoles().map(role => `role/${role}`))
        .concat(MODIFIERS.map(modifier => `modifier/${modifier}`))
        .concat(GENERATED_ARTICLES.map(article => `generated/${article}`)) as WikiArticleLink[];

export function getAllWikiArticles(): WikiArticleLink[] {
    return CONSTANT_ARTICLES.concat(
        Object.keys(customWikiArticles).map(key => key as WikiArticleLink)
    ).concat(
        Object.keys(langJson)
            .filter(key => key.startsWith("wiki.article.standard.") && key.endsWith(".title"))
            .map(key => `standard/${key.split(".")[3]}` as WikiArticleLink)
    );
}


export function getArticleLangKey(page: WikiArticleLink): string {
    const path = page.split('/');


    switch (path[0]) {
        case "role":
            return `role.${path[1]}.name`;
        case "modifier":
            return `wiki.article.modifier.${path[1]}.title`;
        case "category":
            return `wiki.category.${path[1]}`;
        case "standard":
            return `wiki.article.standard.${path[1]}.title`;
        case "generated":
            return `wiki.article.generated.${path[1]}.title`;
        default:
            console.error("Invalid article type: "+path[0]);
            return "ERROR";
    }
}

export function getArticleTitle(page: WikiArticleLink): string {
    const translation = translateChecked(getArticleLangKey(page));
    if (translation === null) {
        // This must be a custom article
        // In which case, the title will be the first line (without any #'s)
        const body = translateCustomWikiArticle(page);
        const title = body.split("\n")[0]?.replace(/#* ?/, "").trim();
        return title;
    }
    return translation;
}

export function wikiPageIsEnabled(
    page: WikiArticleLink,
    enabledRoles: Role[],
    enabledModifiers: ModifierID[],
    wikiDisabledFilter: [string, WikiDisabledFilter] | "default" = "default"
): boolean {
    if (wikiDisabledFilter !== "default") {
        return wikiDisabledFilter[1](page);
    }

    switch (page.split("/")[0]) {
        case "role":
            return enabledRoles.map(role => `role/${role}`).includes(page)
        case "modifier":
            return enabledModifiers.map(modifier => `modifier/${modifier}`).includes(page)
    }

    if (page === "standard/mafia") {
        return enabledRoles.some(role => roleJsonData()[role].roleSets.includes("mafia"))
    } else if (page === "standard/cult") {
        return enabledRoles.some(role => roleJsonData()[role].roleSets.includes("cult"))
    }

    if (page.startsWith("category/")) {
        return partitionWikiPages(getAllWikiArticles(), enabledRoles, enabledModifiers, false)[page.split("/")[1] as any as WikiCategory]
            .filter(p => p !== page)
            .filter(page => wikiPageIsEnabled(page, enabledRoles, enabledModifiers, wikiDisabledFilter))
            .length !== 0
    }

    return true;
}