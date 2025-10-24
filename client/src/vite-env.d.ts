/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WS_ADDRESS: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare module 'virtual:wiki-content' {
    const wikiPages: Record<string, string>;
    export default wikiPages;
}
