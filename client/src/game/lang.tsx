import { WikiArticleLink } from "../components/WikiArticleLink";

export const LANGUAGES: Record<string, Record<string, string>> = Object.fromEntries(
    Object.entries(import.meta.glob("../resources/lang/{*.json,*/lang.json}", { eager: true, import: "default" }))
        .map(([key, value]) => {
            const last = key.split("/").pop()?.replace(".json", "");

            if (last === "lang") {
                return [key.split("/").at(-2), value];
            }

            return [last, value];
        })
);

const ALL_WIKI_ARTICLE_PATHS = import.meta.glob("../resources/lang/**/wiki.article.*.*.md", { eager: true, as: "raw" });

export let langMap: ReadonlyMap<string, string>;
export let langText: string;
export let langJson: Record<string, string>;
export let customWikiArticles: Record<string, string> = {};

switchLanguage("en_us");

export type Language = string;

export function switchLanguage(language: Language) {
    langJson = LANGUAGES[language];
    langMap = new Map<string, string>(Object.entries(langJson));
    langText = JSON.stringify(langJson, null, 1);

    customWikiArticles = {};

    for (const [key, value] of Object.entries(ALL_WIKI_ARTICLE_PATHS)) {
        if (key.startsWith("../resources/lang/" + language + "/")) {
            const fileName = key.split("/").pop()!;
            const articleLink = fileName
                .replace("wiki.article.", "")
                .replace(".md", "")
                .replace(".", "/");

            customWikiArticles[articleLink] = value;
        }
    }

}

/// Returns the translated string with the given key, replacing the placeholders with the given values.
export default function translate(langKey: string, ...valuesList: (string | number)[]): string {
    const translation = translateChecked(langKey, ...valuesList);

    if (translation === null) {
        console.error("Attempted to use non existent lang key: "+langKey);
        return "ERROR: "+langKey;
    } 

    return translation;
}

export function translateCustomWikiArticle(link: WikiArticleLink): string {
    const article = customWikiArticles[link];
    if (!article) {
        console.error("Attempted to use non existent wiki article: "+link);
        return "ERROR: "+link;
    }
    return article;
}

export function translateAny(langKeys: string[], ...valuesList: (string | number)[]): string {
    for (const key of langKeys) {
        const translation = translateChecked(key, ...valuesList);

        if (translation !== null) {
            return translation;
        }
    }

    console.error("Attempted to use non existent lang key: "+langKeys.at(-1));
    return "ERROR: "+langKeys.at(-1);
}

export function translateChecked(langKey: string, ...valuesList: (string | number)[]): string | null {
    let out = langMap.get(langKey);
    if(out===undefined){
        return null;
    }
    for(let i = 0; i < valuesList.length; i++){
        out = out.replace("\\"+(i), valuesList[i] as string);
    }
    return out;
}

export function languageName(language: Language): string {
    const json = LANGUAGES[language];
    return json.language;
}