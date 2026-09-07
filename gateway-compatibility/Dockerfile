FROM gcr.io/distroless/nodejs22-debian13:nonroot@sha256:4e4fb0ce55fd73901600796ef079a9490369d2515d7da31633a91608c82ca13b
WORKDIR /app
COPY gateway-compatibility/server.mjs LICENSE NOTICE ./
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/nodejs/bin/node", "/app/server.mjs"]
