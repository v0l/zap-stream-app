FROM rust:trixie AS builder
WORKDIR /src
COPY . .
RUN ./debian.sh
