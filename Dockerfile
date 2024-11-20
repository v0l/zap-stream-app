FROM rust:bookworm AS BUILDER
WORKDIR /src
COPY . .
RUN ./debian.sh