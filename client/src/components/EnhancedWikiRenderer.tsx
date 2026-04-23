/**
 * Enhanced Wiki Renderer
 * 
 * Renders wiki content with enhanced markdown features:
 * - Custom containers (tip, warning, danger, info)
 * - Dynamic content injection
 * - React component embedding
 * - Auto-linking to other wiki pages
 */

import React, { ReactElement, useMemo } from "react";
import { marked } from "marked";
import StyledText from "./StyledText";
import { WikiPageContent, processEnhancedMarkdown } from "./wikiLoader";
import "./wiki.css";

interface EnhancedWikiRendererProps {
    content: WikiPageContent;
    dynamicData?: Record<string, any>;
}

/**
 * Enhanced Markdown Renderer that supports:
 * - Standard markdown (via marked)
 * - Custom containers (::: tip, ::: warning, etc.)
 * - Variable interpolation {variableName}
 * - Wiki page links [text](page/path)
 */
export default function EnhancedWikiRenderer(props: Readonly<EnhancedWikiRendererProps>): ReactElement {
    const processedContent = useMemo(() => {
        let content = props.content.content;
        
        // Process custom containers
        content = processCustomContainers(content);
        
        // Process variable interpolation
        if (props.dynamicData) {
            content = processVariableInterpolation(content, props.dynamicData);
        }
        
        // Process title variables from metadata
        content = content.replace(/\{title\}/g, props.content.metadata.title);
        
        return content;
    }, [props.content, props.dynamicData]);

    return (
        <div className="enhanced-wiki-content">
            <StyledText markdown={true}>
                {processedContent}
            </StyledText>
        </div>
    );
}

/**
 * Process custom container syntax
 * Converts ::: type content ::: to styled divs
 */
function processCustomContainers(content: string): string {
    const containerRegex = /:::(\s+)?(tip|warning|danger|info|note)([\s\S]*?):::/g;
    
    return content.replace(containerRegex, (match, _space, type, innerContent) => {
        const trimmedContent = innerContent.trim();
        
        // Map container types to icons
        const icons: Record<string, string> = {
            tip: '💡',
            warning: '⚠️',
            danger: '🚫',
            info: 'ℹ️',
            note: '📝'
        };
        
        const icon = icons[type] || '';
        const title = type.charAt(0).toUpperCase() + type.slice(1);
        
        return `<div class="wiki-container wiki-container-${type}">
            <div class="wiki-container-title">${icon} ${title}</div>
            <div class="wiki-container-content">${trimmedContent}</div>
        </div>`;
    });
}

/**
 * Process variable interpolation
 * Replaces {variableName} with actual values from dynamicData
 */
function processVariableInterpolation(content: string, data: Record<string, any>): string {
    return content.replace(/\{(\w+)\}/g, (match, varName) => {
        if (varName in data) {
            const value = data[varName];
            return typeof value === 'string' || typeof value === 'number' ? String(value) : match;
        }
        return match;
    });
}

/**
 * Component for rendering custom wiki containers
 */
export function WikiContainer(props: Readonly<{
    type: 'tip' | 'warning' | 'danger' | 'info' | 'note';
    title?: string;
    children: React.ReactNode;
}>): ReactElement {
    const icons: Record<string, string> = {
        tip: '💡',
        warning: '⚠️',
        danger: '🚫',
        info: 'ℹ️',
        note: '📝'
    };
    
    const icon = icons[props.type] || '';
    const title = props.title || (props.type.charAt(0).toUpperCase() + props.type.slice(1));
    
    return (
        <div className={`wiki-container wiki-container-${props.type}`}>
            <div className="wiki-container-title">{icon} {title}</div>
            <div className="wiki-container-content">
                {props.children}
            </div>
        </div>
    );
}
