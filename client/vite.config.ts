import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'
import checker from 'vite-plugin-checker'

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    checker({
      typescript: true,
      eslint: {
        lintCommand: 'eslint "./src/**/*.{ts,tsx}"',
      },
    }),
  ],
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'build',
    sourcemap: false,
    rollupOptions: {
      // Intercept and handle warnings explicitly
      onwarn(warning, warn) {
        // Throw an error to intentionally crash the build process
        if (true) {
          throw new Error(`[Vite Build Warning]: ${warning.message}`);
        }
      },
    },
  }
}));