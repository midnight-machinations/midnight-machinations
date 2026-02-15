/**
 * Wiki System Integration
 * 
 * This module provides a unified interface for accessing wiki content
 * from both the legacy system (en_us.json) and the new MDX-based system.
 * 
 * It allows for gradual migration of wiki content.
 */

import wikiContentPages from 'virtual:wiki-content';
import { initializeWikiPages, loadWikiPage, hasWikiPage, WikiPageContent } from './wikiLoader';

// Initialize wiki pages from the virtual module
initializeWikiPages(wikiContentPages);

export { loadWikiPage, hasWikiPage };

/**
 * Checks if a wiki article should be loaded from the new MDX system
 * or fall back to the legacy translation system
 */
export function shouldUseNewWikiSystem(articlePath: string): boolean {
    return hasWikiPage(articlePath);
}

/**
 * Get wiki content with automatic fallback to legacy system
 */
export function getWikiContent(articlePath: string): WikiPageContent | null {
    if (hasWikiPage(articlePath)) {
        return loadWikiPage(articlePath);
    }
    
    // Return null to indicate fallback to legacy system
    return null;
}
