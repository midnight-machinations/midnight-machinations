#!/usr/bin/env node
/**
 * Test script to verify wiki content loading
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const wikiDir = path.join(__dirname, 'src', 'resources', 'wiki');

function scanWikiFiles(dir, baseDir = dir, results = []) {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        
        if (entry.isDirectory()) {
            scanWikiFiles(fullPath, baseDir, results);
        } else if (entry.isFile() && (entry.name.endsWith('.mdx') || entry.name.endsWith('.md'))) {
            // Skip README files (consistent with vite plugin)
            if (entry.name.toUpperCase().startsWith('README')) {
                continue;
            }
            
            const relativePath = path.relative(baseDir, fullPath);
            const wikiPath = relativePath
                .replace(/\\/g, '/')
                .replace(/\.(mdx|md)$/, '');
            
            results.push({
                path: wikiPath,
                fullPath: fullPath,
                size: fs.statSync(fullPath).size
            });
        }
    }
    
    return results;
}

console.log('Scanning wiki directory:', wikiDir);
console.log('');

const files = scanWikiFiles(wikiDir);

console.log(`Found ${files.length} wiki files:\n`);

for (const file of files) {
    console.log(`  ✓ ${file.path} (${file.size} bytes)`);
}

console.log('');
console.log('Sample content from first file:');
console.log('━'.repeat(60));

if (files.length > 0) {
    const content = fs.readFileSync(files[0].fullPath, 'utf-8');
    console.log(content.substring(0, 500));
    if (content.length > 500) {
        console.log('... (truncated)');
    }
}

console.log('━'.repeat(60));
console.log('');
console.log('✓ Wiki system test completed successfully!');
