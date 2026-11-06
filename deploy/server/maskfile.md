# Server deployment

Run with `mask --maskfile deploy/server/maskfile.md <task>` from the repository root.

## docker

> Native server image, for self-hosting

### docker build

```sh
cd "$MASKFILE_DIR/../.." && docker build -f deploy/server/Dockerfile -t midnightmachinations-server .
```

### docker run

> Serve on port 9000. Voice settings come from server/.env.local when present

```sh
cd "$MASKFILE_DIR/../.."
env_file=""
[ -f server/.env.local ] && env_file="--env-file server/.env.local"
docker run --rm -p 9000:80 $env_file -e WS_ADDRESS=0.0.0.0:80 midnightmachinations-server
```
