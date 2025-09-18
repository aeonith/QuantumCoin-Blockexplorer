FROM node:20-alpine as build
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
ARG VITE_API
ENV VITE_API=${VITE_API}
RUN npm run build

FROM caddy:2
COPY --from=build /app/dist /usr/share/caddy
EXPOSE 80
