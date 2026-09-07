FROM node:22.23.2-alpine@sha256:c610fcdfb1d5b4740dd70c284ed3cb16bb857e0f7166196e36a5501df7a3aa32 AS build
WORKDIR /app
COPY mcp-scanner/package.json mcp-scanner/package-lock.json ./
RUN npm ci --ignore-scripts
COPY mcp-scanner/tsconfig.json ./
COPY mcp-scanner/src ./src
RUN npm run build && npm prune --omit=dev --ignore-scripts

FROM node:22.23.2-alpine@sha256:c610fcdfb1d5b4740dd70c284ed3cb16bb857e0f7166196e36a5501df7a3aa32
WORKDIR /app
ENV NODE_ENV=production
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist/main.js /app/dist/server.js ./dist/
COPY mcp-scanner/package.json mcp-scanner/package-lock.json LICENSE NOTICE ./
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["node", "/app/dist/main.js"]
