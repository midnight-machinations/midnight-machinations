import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // CLIENT_PORT can be set in .env.local, e.g. to run several git worktrees at once
  const env = loadEnv(mode, process.cwd(), '')

  return {
    plugins: [react()],
    server: {
      port: Number(env.CLIENT_PORT) || 3000,
      open: true
    },
    build: {
      outDir: 'build',
      sourcemap: false
    }
  }
})
