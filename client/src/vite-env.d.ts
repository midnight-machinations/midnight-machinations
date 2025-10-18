/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WS_ADDRESS: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
