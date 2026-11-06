# Deploying the client

The client is a static Vite build (`client/build`, with files from `assets/public`). The game server address is compiled in through `VITE_WS_ADDRESS`.

## Cloudflare

`wrangler.json` here serves `client/build` as [Workers static assets](https://developers.cloudflare.com/workers/static-assets/), with single-page-app fallback. Static asset requests are free and unlimited on the free plan.

```sh
mask --maskfile deploy/client/maskfile.md deploy wss://midnight-machinations-server.<account>.workers.dev
```

Wrangler is borrowed from `voice/node_modules`, so the client needs no extra dependencies. Add a custom domain or route in `wrangler.json` (or the dashboard) when you want one.

## Docker

```sh
mask --maskfile deploy/client/maskfile.md docker build wss://example.com
mask --maskfile deploy/client/maskfile.md docker run   # http://localhost:3000
```
