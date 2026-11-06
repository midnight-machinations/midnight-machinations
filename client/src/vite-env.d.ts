/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WS_ADDRESS: string
  readonly VITE_FEATURE_VOICE?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
