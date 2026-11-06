import { defineConfig, devices } from "@playwright/test";

// `mask config` writes this checkout's client port (it differs per worktree) to client/.env.local.
try {
    process.loadEnvFile(new URL("./client/.env.local", import.meta.url));
} catch {
    // Not generated yet, so fall back to the default port.
}

const port = Number(process.env.CLIENT_PORT) || 3000;
const baseURL = process.env.BASE_URL ?? `http://localhost:${port}`;

export default defineConfig({
    testDir: "e2e",
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    reporter: process.env.CI ? "github" : "list",
    use: { baseURL, trace: "on-first-retry" },
    projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
    // Starts the client dev server unless one is already running or BASE_URL points elsewhere.
    // Tests that need the game server should also start it here (see `mask server run`).
    webServer: process.env.BASE_URL
        ? undefined
        : {
              command: `pnpm --dir client dev --no-open --strictPort --port ${port}`,
              url: baseURL,
              reuseExistingServer: true,
              timeout: 120_000,
          },
});
