/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WS_ADDRESS: string
  readonly VITE_TURN_USERNAME: string
  readonly VITE_TURN_CREDENTIAL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
