# Wiki Content System

This directory contains wiki content for Midnight Machinations in an enhanced markdown format.

## Overview

The new wiki system provides:

1. **Enhanced Markup**: Markdown with custom extensions for better content formatting
2. **Maintainability**: Individual files instead of a large JSON file
3. **Translation Support**: Frontmatter metadata supports translation
4. **Dynamic Content**: JavaScript-enabled pages for auto-updating content

## Directory Structure

```
wiki/
├── standard/       # Standard game mechanics articles
├── role/          # Role-specific articles (future use)
├── modifier/      # Modifier articles (future use)
├── category/      # Category overview pages (future use)
└── generated/     # Dynamic/generated content pages
```

## File Format

Wiki pages use MDX format (Markdown with extended features):

### Basic Example

```markdown
---
title: "Article Title"
titleVariants:
  - "Alternative Title 1"
  - "Alternative Title 2"
category: "standard"
translatable: true
---

# {title}

Your content here with **markdown** formatting.

## Subheading

More content...
```

### Enhanced Features

#### 1. Custom Containers

Add styled information boxes:

```markdown
::: tip
This is a helpful tip for players!
:::

::: warning
Be careful about this mechanic!
:::

::: danger
This is very important!
:::

::: info
Additional information here.
:::

::: note
A general note.
:::
```

#### 2. Variable Interpolation

Use dynamic content:

```markdown
The game has {roleSetCount} role sets.
```

#### 3. Wiki Links

Link to other wiki pages using standard markdown links:

```markdown
See the [Godfather](role/godfather) role for more info.
Learn about [Whispering](standard/whisper).
```

#### 4. Dynamic JavaScript Content (Advanced)

For pages that need to generate content dynamically:

```markdown
---
title: "Role Sets"
dynamic: true
---

<script>
export function getRoleSets() {
  return window.GAME_MANAGER?.roleSetData || [];
}
</script>

# {title}

<DynamicRoleSetList />
```

## Migration Guide

### From en_us.json to MDX

Old format (in `en_us.json`):
```json
{
  "wiki.article.standard.whisper.title": "Whisper",
  "wiki.article.standard.whisper.text": "Content here..."
}
```

New format (`wiki/standard/whisper.mdx`):
```markdown
---
title: "Whisper"
category: "standard"
translatable: true
---

# {title}

Content here...
```

### Benefits of Migration

1. **Easier Editing**: Plain markdown is easier to write and read than JSON strings
2. **Version Control**: Individual files show clearer diffs in Git
3. **Rich Formatting**: Use all markdown features plus custom extensions
4. **No Escaping**: No need to escape newlines or quotes
5. **Dynamic Updates**: JavaScript can auto-update content when roles are added

## Translation Support

The system maintains translation support through:

1. **Frontmatter metadata**: Title and variants can be translated
2. **Content files per language**: Create language-specific versions (e.g., `whisper.en.mdx`, `whisper.es.mdx`)
3. **Variable interpolation**: Dynamic content uses the current language

### Future: Multi-language Support

To add support for additional languages:

```
wiki/
├── en/              # English content
│   ├── standard/
│   └── role/
├── es/              # Spanish content
│   ├── standard/
│   └── role/
└── ...
```

## Styling

Custom container styles are defined in `components/wiki.css`:

- `.wiki-container`: Base container styling
- `.wiki-container-tip`: Green tip boxes
- `.wiki-container-warning`: Orange warning boxes
- `.wiki-container-danger`: Red danger boxes
- `.wiki-container-info`: Blue info boxes
- `.wiki-container-note`: Gray note boxes

## Build Process

The wiki content is processed at build time by `vite-plugin-wiki-content.ts`:

1. Scans `src/resources/wiki/` directory
2. Loads all `.mdx` and `.md` files
3. Generates a virtual module with all content
4. Enables hot-reload during development

## Usage in Code

```typescript
import { getWikiContent, shouldUseNewWikiSystem } from './wikiIntegration';

// Check if article exists in new system
if (shouldUseNewWikiSystem('standard/whisper')) {
  const content = getWikiContent('standard/whisper');
  // Use content
}
```

## Adding New Articles

1. Create a new `.mdx` file in the appropriate directory
2. Add frontmatter with metadata
3. Write content using enhanced markdown
4. The build system will automatically include it

Example:

```bash
# Create a new article
touch src/resources/wiki/standard/my-new-article.mdx

# Edit the file
# The article will be available at 'standard/my-new-article'
```

## Best Practices

1. **Keep frontmatter simple**: Only include necessary metadata
2. **Use semantic containers**: Choose the right container type for the content
3. **Link related pages**: Help readers navigate to related topics
4. **Test locally**: Run `pnpm dev` to see changes immediately
5. **Gradual migration**: Migrate articles one at a time, testing each

## Future Enhancements

Potential future improvements:

- [ ] Full MDX component support
- [ ] Per-language content directories
- [ ] Auto-generation of category pages
- [ ] Search indexing of MDX content
- [ ] Table of contents generation
- [ ] Embedded React components for interactive examples
