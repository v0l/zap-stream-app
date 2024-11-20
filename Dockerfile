FROM rust:bookworm AS builder
WORKDIR /src
COPY . .
RUN ./debian.sh