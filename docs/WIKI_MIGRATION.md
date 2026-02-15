# Migration Guide: Moving Wiki Content from en_us.json to MDX

This guide helps you migrate wiki articles from the legacy JSON format to the new MDX format.

## Quick Reference

| Aspect | Old System (en_us.json) | New System (MDX files) |
|--------|------------------------|------------------------|
| **Location** | `client/src/resources/lang/en_us.json` | `client/src/resources/wiki/` |
| **Format** | JSON key-value pairs | MDX files with frontmatter |
| **Editing** | Edit JSON strings | Edit markdown files |
| **Links** | Manual keyword system | Standard markdown links |
| **Formatting** | Escape newlines as `\n` | Natural markdown |
| **Translation** | All in same file | Metadata + future multi-language support |

## Step-by-Step Migration

### Step 1: Identify the Article

Find the article in `en_us.json`:

```json
"wiki.article.standard.example.title": "Example Article",
"wiki.article.standard.example.text": "Content goes here...",
```

### Step 2: Create the MDX File

Determine the file location based on article type:

- `standard/*` → `wiki/standard/*.mdx`
- `modifier/*` → `wiki/modifier/*.mdx`
- `role/*` → `wiki/role/*.mdx`
- `category/*` → `wiki/category/*.mdx`

Example: `wiki.article.standard.example` becomes `wiki/standard/example.mdx`

### Step 3: Convert the Content

#### Basic Template

```markdown
---
title: "Article Title"
category: "standard"
translatable: true
---

# {title}

Article content here...
```

#### With Title Variants

If the old system has variants like:

```json
"wiki.article.standard.whisper.title": "Whisper",
"wiki.article.standard.whisper.title:var.0": "Whispers",
"wiki.article.standard.whisper.title:var.1": "Whispering",
```

Convert to:

```markdown
---
title: "Whisper"
titleVariants:
  - "Whispers"
  - "Whispering"
  - "Whispered"
category: "standard"
---
```

### Step 4: Format the Content

#### Replace Newlines

Old:
```json
"text": "Line 1\nLine 2\n- Bullet 1\n- Bullet 2"
```

New:
```markdown
Line 1
Line 2
- Bullet 1
- Bullet 2
```

#### Convert Links

Old (keyword system):
```json
"See the godfather role"
```

New (markdown links):
```markdown
See the [Godfather](role/godfather) role
```

#### Add Custom Containers

Enhance the content with styled containers:

```markdown
::: tip
This is helpful information!
:::

::: warning
Be careful about this!
:::
```

## Example Migrations

See the existing MDX files in `client/src/resources/wiki/standard/` for complete examples:

- `mafia.mdx` - Simple article with links
- `whisper.mdx` - Article with containers
- `aura.mdx` - Complex article with multiple sections
- `defense.mdx` - Article with role references

## Best Practices

1. **Migrate gradually**: Start with a few articles, test thoroughly
2. **Enhance content**: Use this as an opportunity to improve articles
3. **Add containers**: Make important info stand out with tip/warning boxes
4. **Link related content**: Help users navigate between related articles
5. **Test in-game**: Always verify the article renders correctly in the actual game

## Validation Checklist

After migrating an article:

- [ ] File is in correct directory
- [ ] Frontmatter is properly formatted
- [ ] Title is correct
- [ ] Links work (test in browser)
- [ ] Custom containers render correctly
- [ ] No formatting errors
- [ ] Fallback to old system works if needed
- [ ] Article appears in wiki search

## Getting Help

- Check `client/src/resources/wiki/README.md` for detailed system documentation
- Look at existing MDX files for examples
- Test changes with `pnpm dev`
