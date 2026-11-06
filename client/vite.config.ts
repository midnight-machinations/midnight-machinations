import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // CLIENT_PORT can be set in .env.local, e.g. to run several git worktrees at once
  const env = loadEnv(mode, process.cwd(), '')

  return {
    plugins: [react()],
    // Static files shared with other packages live in the root assets folder.
    publicDir: '../assets/public',
    // AudioWorklet has import.meta.url but no WorkerGlobalScope/self.location.
    worker: { format: 'es' },
    server: {
      port: Number(env.CLIENT_PORT) || 3000,
      open: true,
      // Roles, language files, and fonts are imported from the root assets folder.
      fs: { allow: ['..'] }
    },
    build: {
      outDir: 'build',
      sourcemap: false
    }
  }
})
