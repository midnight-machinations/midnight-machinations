import { translateChecked } from "../game/lang";
import { ModifierID } from "../game/modifiers";
import { Role } from "../game/roleState.d";
import { WikiArticleLink } from "./WikiArticleLink";


export function getArticleTooltip(page: WikiArticleLink): string | null {
    if (page.startsWith("role/")) {
        const role = page.split("/")[1] as Role;
        return translateChecked(`wiki.article.role.${role}.reminder`);
    } else if (page.startsWith("modifier/")) {
        const modifier = page.split("/")[1] as ModifierID;
        return translateChecked(`wiki.article.modifier.${modifier}.text`);
    } else if (page.startsWith("standard/")) {
        const standard = page.split("/")[1];
        return translateChecked(`wiki.article.standard.${standard}.text`);
    } else if (page.startsWith("generated/")) {
        return null;
    } else if (page.startsWith("category/")) {
        const category = page.split("/")[1];
        return translateChecked(`wiki.category.${category}.text`);
    }
    return null;
}
