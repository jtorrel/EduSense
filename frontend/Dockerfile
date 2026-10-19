# Image du frontend. Contexte de build : frontend/.
#   docker build -t edusense/frontend frontend

FROM node:24-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# Fichiers statiques servis par nginx en utilisateur non root (port 8080).
FROM nginxinc/nginx-unprivileged:1.29-alpine
COPY security-headers.conf /etc/nginx/snippets/security-headers.conf
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 8080
