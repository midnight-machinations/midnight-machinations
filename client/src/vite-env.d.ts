/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WS_ADDRESS: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare module 'virtual:wiki-content' {
    interface WikiPageMetadata {
        title: string;
        titleVariants?: string[];
        category: 'standard' | 'role' | 'modifier' | 'category' | 'generated';
        translatable?: boolean;
        dynamic?: boolean;
        tags?: string[];
    }
    
    interface WikiPageContent {
        metadata: WikiPageMetadata;
        content: string;
        rawContent: string;
    }
    
    const wikiPages: Record<string, WikiPageContent>;
    export default wikiPages;
}
