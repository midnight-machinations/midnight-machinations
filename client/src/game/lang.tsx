import { Parser, Expression } from "expr-eval"

export let langMap: ReadonlyMap<string, string>;
export let langText: string;
export let langJson: any;

export const LANGUAGES = ["en_us", "broken_keyboard", "dyslexic"] as const;
export type Language = typeof LANGUAGES[number]
switchLanguage("en_us");

export function switchLanguage(language: Language) {
    langJson = require("../resources/lang/" + language + ".json");
    langMap = new Map<string, string>(Object.entries(langJson));
    langText = JSON.stringify(langJson, null, 1);
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
    if (langMap.get(langKey + ':cases') !== undefined) {
        // These expensive calculations are only run when things have special cases
        for (const [caseKey, caseValue] of 
            [...langMap.entries()].filter(([key]) => key.startsWith(langKey + ":case:"))
        ) {
            const caseExpr = caseKey.substring((langKey + ":case:").length);
            if (Parser.parse(populateValues(caseExpr, valuesList)).evaluate()) {
                return populateValues(caseValue, valuesList);
            }
        }
    }

    let out = langMap.get(langKey);
    if(out===undefined){
        return null;
    }
    return populateValues(out, valuesList);
}

function populateValues(text: string, valuesList: (string | number)[]): string {
    for (let i = 0; i < valuesList.length; i++) {
        text = text.replace("\\" + i, valuesList[i] as string);
    }
    return text;
}

export function languageName(language: Language): string {
    const json = require("../resources/lang/" + language + ".json");
    return json.language;
}