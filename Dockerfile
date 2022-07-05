# Build container

FROM rust:1.62-alpine3.16 as build

WORKDIR /home

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo update

COPY src src

RUN cargo build --release

# Execution container

FROM python:3.7-alpine3.16

RUN pip install flake8==4.0.1

COPY --from=build /home/target/release/github-actions-flake8 /usr/bin/github-actions-flake8

WORKDIR /github/workspace

CMD [ "github-actions-flake8" ]
