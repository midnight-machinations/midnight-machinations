/**
 * Wiki Content Loader
 * 
 * This module handles loading wiki content that has been pre-parsed at build time.
 * It supports:
 * - Frontmatter metadata (parsed at build time)
 * - Enhanced markdown with custom extensions
 * - Dynamic content with JavaScript
 * - Translation support
 */

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
 * Register a wiki page (data comes pre-parsed from build time)
 * @param pagePath - The page path
 * @param content - The pre-parsed page content
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

// Initialize wiki pages - data comes pre-parsed from the build system
export function initializeWikiPages(pages: Record<string, WikiPageContent>): void {
    for (const [path, content] of Object.entries(pages)) {
        registerWikiPage(path, content);
    }
}
