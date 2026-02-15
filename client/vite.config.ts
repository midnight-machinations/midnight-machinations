import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wikiContentPlugin from './vite-plugin-wiki-content'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    wikiContentPlugin()
  ],
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'build',
    sourcemap: false
  }
})
