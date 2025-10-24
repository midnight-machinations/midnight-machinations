/**
 * Wiki Content Loader
 * 
 * This module handles loading and parsing wiki content from MDX files.
 * It supports:
 * - Frontmatter parsing for metadata
 * - Enhanced markdown with custom extensions
 * - Dynamic content with JavaScript
 * - Translation support
 */

import matter from 'gray-matter';

export interface WikiPageMetadata {
    title: string;
    titleVariants?: string[];
    category: 'standard' | 'role' | 'modifier' | 'category' | 'generated';
    translatable?: boolean;
    dynamic?: boolean;
    tags?: string[];
}

export interface WikiPageContent {
    metadata: WikiPageMetadata;
    content: string;
    rawContent: string;
}

// This will be populated at build time with all wiki pages
const wikiPages = new Map<string, WikiPageContent>();

/**
 * Load a wiki page by its path
 * @param pagePath - The wiki page path (e.g., "standard/mafia", "role/godfather")
 * @returns The wiki page content or null if not found
 */
export function loadWikiPage(pagePath: string): WikiPageContent | null {
    return wikiPages.get(pagePath) || null;
}

/**
 * Check if a wiki page exists
 * @param pagePath - The wiki page path
 * @returns True if the page exists
 */
export function hasWikiPage(pagePath: string): boolean {
    return wikiPages.has(pagePath);
}

/**
 * Get all available wiki pages
 * @returns Array of all wiki page paths
 */
export function getAllWikiPages(): string[] {
    return Array.from(wikiPages.keys());
}

/**
 * Parse MDX content with frontmatter
 * @param mdxContent - Raw MDX content with frontmatter
 * @returns Parsed wiki page content
 */
export function parseMDXContent(mdxContent: string): WikiPageContent {
    const { data, content } = matter(mdxContent);
    
    return {
        metadata: data as WikiPageMetadata,
        content: content,
        rawContent: mdxContent
    };
}

/**
 * Register a wiki page (called at build time)
 * @param pagePath - The page path
 * @param content - The page content
 */
export function registerWikiPage(pagePath: string, content: WikiPageContent): void {
    wikiPages.set(pagePath, content);
}

/**
 * Enhanced markdown extensions for wiki content
 * These add custom syntax on top of standard markdown:
 * - ::: for custom containers (tip, warning, danger, etc.)
 * - {variable} for dynamic content injection
 * - <Component /> for React components
 */

export interface MarkdownExtension {
    name: string;
    pattern: RegExp;
    render: (match: RegExpMatchArray, content: string) => string;
}

export const markdownExtensions: MarkdownExtension[] = [
    // Custom container syntax: ::: type ... :::
    {
        name: 'container',
        pattern: /:::(\s+)?(\w+)([\s\S]*?):::/g,
        render: (match, content) => {
            const type = match[2] || 'info';
            const innerContent = match[3]?.trim() || '';
            return `<div class="wiki-container wiki-container-${type}">${innerContent}</div>`;
        }
    },
    // Variable interpolation: {variableName}
    {
        name: 'variable',
        pattern: /\{(\w+)\}/g,
        render: (match) => {
            const varName = match[1];
            return `<span data-wiki-var="${varName}"></span>`;
        }
    }
];

/**
 * Process enhanced markdown content
 * @param content - Raw markdown content
 * @returns Processed content with extensions applied
 */
export function processEnhancedMarkdown(content: string): string {
    let processed = content;
    
    for (const extension of markdownExtensions) {
        processed = processed.replace(extension.pattern, (...args) => {
            const match = args;
            return extension.render(match as any, processed);
        });
    }
    
    return processed;
}

// Initialize with empty pages - these will be populated by the build system or at runtime
export function initializeWikiPages(pages: Record<string, string>): void {
    for (const [path, content] of Object.entries(pages)) {
        const parsed = parseMDXContent(content);
        registerWikiPage(path, parsed);
    }
}
