# Fix for Buffer Error in Wiki System

## Issue

When navigating to the website, users encountered the following error:

```
Uncaught ReferenceError: Buffer is not defined
    matter gray-matter.js:3171
    matter gray-matter.js:3340
    matter index.js:34
    parseMDXContent wikiLoader.ts:64
    initializeWikiPages wikiLoader.ts:139
```

This error caused a white screen on the website.

## Root Cause

The `gray-matter` library was being imported and used in browser-side code (`wikiLoader.ts`). `gray-matter` is a Node.js library that depends on the `Buffer` API, which is not available in browser environments.

The original implementation had:
1. Vite plugin reading raw MDX files
2. Passing raw content to browser
3. Browser code parsing frontmatter with `gray-matter` at runtime
4. This caused the Buffer error

## Solution

Moved frontmatter parsing from runtime (browser) to build time (Vite plugin):

### Changes Made

1. **`vite-plugin-wiki-content.ts`**
   - Parse MDX frontmatter at build time using `gray-matter`
   - Generate pre-parsed content with metadata already extracted
   - Pass structured data to browser instead of raw strings

2. **`wikiLoader.ts`**
   - Removed `gray-matter` import
   - Removed `parseMDXContent` function
   - Updated `initializeWikiPages` to accept pre-parsed content
   - Now only manages pre-parsed wiki pages

3. **`vite-env.d.ts`**
   - Updated virtual module type definition
   - Changed from `Record<string, string>` to `Record<string, WikiPageContent>`
   - Added metadata type definitions

## Benefits

1. **No Runtime Errors**: Buffer dependency removed from browser code
2. **Better Performance**: Parsing happens once at build time instead of on every page load
3. **Smaller Bundle**: Removed gray-matter library from browser bundle
   - Before: 1,355.81 KB
   - After: 1,217.98 KB
   - Reduction: 137.83 KB (10% smaller!)
4. **Cleaner Architecture**: Clear separation between build-time and runtime concerns

## Verification

- ✅ Build passes successfully (2.60s)
- ✅ No gray-matter in browser bundle
- ✅ Metadata correctly embedded
- ✅ No Buffer errors
- ✅ Bundle size reduced by 10%

## Commit

Fixed in commit `7b13dd6`
