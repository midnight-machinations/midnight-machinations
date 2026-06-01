import { ReactElement } from "react";
import { translateChecked } from "../game/lang";
import { ModifierID } from "../game/modifiers";
import { Role } from "../game/roleState.d";
import { WikiArticleLink } from "./WikiArticleLink";
import StyledText, { DUMMY_NAMES_KEYWORD_DATA, DUMMY_ROLE_LIST_KEYWORD_DATA } from "./StyledText";

export default function WikiArticleTooltip(props: Readonly<{
    tooltip: string | null
}>): ReactElement | null {
    if (!props.tooltip) return null;

    return <div className="wiki-article wiki-article-tooltip">
        <StyledText
            noLinks={true}
            markdown={true}
            playerKeywordData={DUMMY_NAMES_KEYWORD_DATA}
            roleListKeywordData={DUMMY_ROLE_LIST_KEYWORD_DATA}
        >
            {props.tooltip}
        </StyledText>
    </div>
}


export function getArticleTooltip(page: WikiArticleLink): string | null {
    const tooltipText = getArticleTooltipText(page);
    // Max 300 characters
    const shortened = tooltipText?.substring(0, 300);
    // Max 3 lines
    const truncated = shortened?.split("\n").slice(0, 3).join("\n");
    // Add ellipsis
    const hasMore = tooltipText && truncated && truncated.length < tooltipText.length;
    const ellipsized = hasMore ? (truncated + "...") : tooltipText;
    return ellipsized;
}

function getArticleTooltipText(page: WikiArticleLink): string | null {
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