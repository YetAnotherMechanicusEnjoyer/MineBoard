FROM rust:1.88.0-slim-bookworm AS builder-back

WORKDIR /app

RUN apt-get update && \
  apt-get install -y \
  pkg-config \
  libssl-dev \
  build-essential && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src/ ./src/

RUN cargo build --release --locked

FROM node:22.0-bookworm-slim AS builder-front

WORKDIR /app

COPY ./front .

RUN npm i && npm run build

FROM openjdk:21-rc-jdk-slim-bookworm

RUN apt-get update && \
  apt-get install -y \
  libssl-dev \
  && apt-get clean

WORKDIR /app

COPY --from=builder-back /app/target/release/mineboard /app/mineboard
COPY --from=builder-front /app/dist /app/front/dist/
COPY ./server/ /app/server/

RUN chmod +x ./mineboard

CMD [ "./mineboard" ]

