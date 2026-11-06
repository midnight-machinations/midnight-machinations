# Native game server, for self-hosting outside Cloudflare.
# Build from the repository root: docker build -f deploy/server/Dockerfile -t midnightmachinations-server .
FROM rust:1.89.0-slim-bookworm AS build
WORKDIR /app
COPY assets ./assets
COPY server ./server
RUN cd server && cargo build --release --bin mafia_server

FROM debian:bookworm-slim
COPY --from=build /app/server/target/release/mafia_server /mafia_server
ENV WS_ADDRESS=0.0.0.0:80
EXPOSE 80
CMD ["/mafia_server"]
